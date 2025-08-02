use std::time::{Duration, Instant};

use log::error;

use crate::config::WorkDefinition;
use crate::models::{BlockStatus, WorkStep};
use crate::runner::service_worker::{
    AsyncOperationStatus, CtrlOutputWriter,
};
use crate::runner::service_worker::async_operation::create_cmd;
use crate::runner::service_worker::block_worker::BlockWorker;
use crate::runner::service_worker::req_checker::RequirementChecker;
use crate::system_state::OperationType;
use crate::utils::format_err;

pub trait WorkHandler {
    fn handle_work(&self);
}

impl WorkHandler for BlockWorker {
    fn handle_work(&self) {
        let work_dir = self.query_service(|service| service.definition.dir.clone());
        let block_status = self.get_block_status();
        let check_status = self.get_operation_status(OperationType::Check);
        let work_status = self.get_operation_status(OperationType::Work);

        let (step, skip_if_healthy) = match block_status {
            BlockStatus::Working {
                step,
                skip_if_healthy,
            } => (step, skip_if_healthy),
            _ => {
                error!("ERROR: invoked work-processing function with invalid block status {block_status:?}");
                return;
            }
        };

        let skip_if_healthy = skip_if_healthy && self.query_block(|block| match block.work {
            // Command sequences are skippable if necessary
            WorkDefinition::CommandSeq { .. } => true,
            // Processes are not, since a healthy process-block must have the handle of the process it has spawned
            WorkDefinition::Process { .. } => false,
        });

        // FIXME skipping should not be possible for process-type work?

        match step {
            // Ensure that there's no lingering process. There should not be if other actions are handled correctly,
            // but some defensive programming here doesn't hurt.
            WorkStep::Initial => {
                self.stop_all_operations_and_then(|| {
                    self.update_status(
                        BlockStatus::Working {
                            skip_if_healthy,
                            step: WorkStep::PrerequisiteCheck {
                                checks_completed: 0,
                                last_failure: None,
                            },
                        }
                    );
                });
            }

            WorkStep::PrerequisiteCheck {
                last_failure: Some(failure_time),
                ..
            } if Instant::now().duration_since(failure_time) < PRE_REQ_FAILURE_WAIT => {
                // Intentionally empty: we've checked prerequisites recently and failed, so just hold on and wait until
                // an appropriate time has elapsed. The prereqs will be checked again in a future iteration.
            }

            WorkStep::PrerequisiteCheck {
                checks_completed,
                ..
            } => {
                let current_requirement = self.query_block(|block| {
                    block.prerequisites.get(checks_completed).map(|req| req.clone())
                });

                match (check_status, current_requirement) {
                    (_, None) if skip_if_healthy => {
                        self.update_status(
                            BlockStatus::Working {
                                skip_if_healthy,
                                step: WorkStep::PreWorkHealthCheck {
                                    checks_completed: 0,
                                },
                            },
                        );
                    }
                    (_, None) => {
                        self.update_status(
                            BlockStatus::Working {
                                skip_if_healthy,
                                step: WorkStep::PerformWork {
                                    steps_completed: 0
                                },
                            },
                        );
                    }
                    (None, Some(requirement)) => {
                        self.check_requirement(&requirement);
                    }
                    (Some(AsyncOperationStatus::Failed), _) => {
                        self.clear_stopped_operation(OperationType::Check);
                        self.update_status(
                            BlockStatus::Working {
                                skip_if_healthy,
                                step: WorkStep::PrerequisiteCheck {
                                    checks_completed: 0,
                                    last_failure: Some(Instant::now()),
                                },
                            },
                        );
                    }
                    (Some(AsyncOperationStatus::Ok), _) => {
                        // Increment the amount of successful checks
                        self.clear_stopped_operation(OperationType::Check);
                        self.update_status(
                            BlockStatus::Working {
                                skip_if_healthy,
                                step: WorkStep::PrerequisiteCheck {
                                    checks_completed: checks_completed + 1,
                                    last_failure: None,
                                },
                            },
                        );
                    }
                    (Some(AsyncOperationStatus::Running), _) => {
                        // Do nothing, wait for the async check to finish
                    }
                }
            }

            WorkStep::PreWorkHealthCheck { checks_completed } => {
                let current_requirement = self.query_block(|block| {
                    block.health.requirements.get(checks_completed).map(|req| req.clone())
                });
                let has_health_checks = self.query_block(|block| !block.health.requirements.is_empty());

                match (check_status, current_requirement) {
                    // If the block has no health checks then we must not treat "all requirements passed" as a free
                    // ticket to skip work, but we must always execute the blocks work.
                    (_, None) if !has_health_checks => self.update_status(
                        BlockStatus::Working {
                            skip_if_healthy,
                            step: WorkStep::PerformWork {
                                steps_completed: 0,
                            },
                        },
                    ),
                    // Otherwise, if there is no current requirement then we know that all of them have been
                    // successfully checked
                    (_, None) => self.update_status(
                        BlockStatus::Ok,
                    ),
                    // No ongoing process, still requirements to check => start the check for the next one
                    (None, Some(requirement)) => {
                        self.check_requirement(&requirement);
                    }
                    // Health check failed, we must fully perform the block's work. Move into the appropriate state
                    (Some(AsyncOperationStatus::Failed), _) => {
                        self.clear_stopped_operation(OperationType::Check);
                        self.update_status(
                            BlockStatus::Working {
                                skip_if_healthy,
                                step: WorkStep::PerformWork {
                                    steps_completed: 0,
                                },
                            },
                        );
                    }
                    (Some(AsyncOperationStatus::Ok), _) => {
                        // Increment the amount of successful checks
                        self.clear_stopped_operation(OperationType::Check);
                        self.update_status(
                            BlockStatus::Working {
                                skip_if_healthy,
                                step: WorkStep::PreWorkHealthCheck {
                                    checks_completed: checks_completed + 1,
                                },
                            },
                        );
                    }
                    (Some(AsyncOperationStatus::Running), _) => {
                        // Do nothing, wait for the async check to finish
                    }
                }
            }

            WorkStep::PerformWork { steps_completed } => {
                match self.query_block(|block| block.work.clone()) {
                    WorkDefinition::CommandSeq { commands: executable_entries } => {
                        let next_executable = executable_entries.get(steps_completed);

                        match (work_status, next_executable) {
                            // Not performing any work, no more work to perform => move to post-work health check
                            (_, None) => self.update_status(
                                BlockStatus::Working {
                                    skip_if_healthy,
                                    step: WorkStep::PostWorkHealthCheck {
                                        start_time: Instant::now(),
                                        checks_completed: 0,
                                        last_failure: None,
                                    }
                                },
                            ),
                            // No ongoing process, but some work left to perform. Attempt to spawn a child process
                            (None, Some(executable)) => {
                                let mut command = create_cmd(
                                    executable,
                                    Some(work_dir),
                                );
                                self.add_ctrl_output(format!("Exec: {executable}"));

                                match command.spawn() {
                                    Ok(process_handle) => {
                                        self.register_external_work(process_handle, OperationType::Work);
                                    }
                                    Err(error) => {
                                        self.update_status(BlockStatus::Error);
                                        self.add_ctrl_output(format_err!("Failed to spawn child process", error));
                                    }
                                }
                            }
                            // A work operation has failed. Move into error state
                            (Some(AsyncOperationStatus::Failed), _) => {
                                self.clear_stopped_operation(OperationType::Work);
                                self.update_status(BlockStatus::Error);
                            }
                            (Some(AsyncOperationStatus::Ok), _) => {
                                // Increment the number of commands successfully completed
                                self.clear_stopped_operation(OperationType::Work);
                                self.update_status(
                                    BlockStatus::Working {
                                        skip_if_healthy,
                                        step: WorkStep::PerformWork {
                                            steps_completed: steps_completed + 1,
                                        },
                                    },
                                );
                            }
                            (Some(AsyncOperationStatus::Running), _) => {
                                // Do nothing, wait for the current work to finish
                            }

                        }
                    }
                    WorkDefinition::Process { executable } => {
                        let mut command = create_cmd(
                            &executable,
                            Some(work_dir),
                        );
                        self.add_ctrl_output(format!("Exec: {executable}"));

                        match command.spawn() {
                            Ok(process_handle) => {
                                // Process launched successfully, move to post-work health check
                                self.register_external_work(process_handle, OperationType::Work);
                                self.update_status(BlockStatus::Working {
                                    skip_if_healthy,
                                    step: WorkStep::PostWorkHealthCheck {
                                        start_time: Instant::now(),
                                        checks_completed: 0,
                                        last_failure: None,
                                    }
                                })
                            }
                            Err(error) => {
                                self.update_status(BlockStatus::Error);
                                self.add_ctrl_output(format_err!("Failed to spawn child process", error));
                            }
                        }
                    }
                }
            }

            WorkStep::PostWorkHealthCheck { start_time, checks_completed, last_failure } => {
                let current_requirement = self.query_block(|block| {
                    block.health.requirements.get(checks_completed).map(|req| req.clone())
                });
                let timeout = self.query_block(|block| block.health.timeout.clone());

                match (check_status, current_requirement, last_failure) {
                    // If there are no more (or at all) requirements to check, then we can finally consider the
                    // block healthy
                    (_, None, _) => self.update_status(BlockStatus::Ok),
                    // We have failed at least one health check and have exceeded the timout.
                    (_, _, Some(_)) if Instant::now().duration_since(start_time) > timeout => {
                        // TODO should this kill the process or not?
                        self.stop_all_operations_and_then(|| self.update_status(BlockStatus::Error));
                    }
                    (_, _, Some(failure_time)) if Instant::now().duration_since(failure_time) < POST_WORK_HEALTH_FAILURE_WAIT => {
                        // We have failed at least once a short time ago. Just wait for time to elapse so that we dont
                        // spam health checks constantly
                    }
                    // No ongoing check, still have requirements to check => start the next check
                    (None, Some(requirement), _) => {
                        self.check_requirement(&requirement);
                    }
                    // Health check failed, we must fully perform the block's work. Move into the appropriate state
                    (Some(AsyncOperationStatus::Failed), _, _) => {
                        self.clear_stopped_operation(OperationType::Check);
                        self.update_status(
                            BlockStatus::Working {
                                skip_if_healthy,
                                step: WorkStep::PostWorkHealthCheck {
                                    start_time,
                                    checks_completed: 0,
                                    last_failure: Some(Instant::now()),
                                },
                            },
                        );
                    }
                    // Current check is successful, increment the amount of successful checks
                    (Some(AsyncOperationStatus::Ok), _, _) => {
                        // Increment the amount of successful checks
                        self.clear_stopped_operation(OperationType::Check);
                        self.update_status(
                            BlockStatus::Working {
                                skip_if_healthy,
                                step: WorkStep::PostWorkHealthCheck {
                                    start_time,
                                    last_failure,
                                    checks_completed: checks_completed + 1,
                                },
                            },
                        );
                    }
                    (Some(AsyncOperationStatus::Running), _, _) => {
                        // Do nothing, wait for the async check to finish
                    }
                }
            }
        }
    }
}

const PRE_REQ_FAILURE_WAIT: Duration = Duration::from_millis(500);
const POST_WORK_HEALTH_FAILURE_WAIT: Duration = Duration::from_millis(3000);