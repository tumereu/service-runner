use std::time::{Duration, Instant};

use log::{debug, error};

use crate::config::WorkDefinition;
use crate::models::{BlockStatus, WorkStep};
use crate::runner::service_worker::{
    AsyncOperationStatus, CtrlOutputWriter,
};
use crate::runner::service_worker::async_operation::create_cmd;
use crate::runner::service_worker::service_block_context::ServiceBlockContext;
use crate::runner::service_worker::req_checker::{RequirementCheckResult, RequirementChecker};
use crate::system_state::OperationType;
use crate::utils::format_err;

pub trait WorkHandler {
    fn handle_work(&self);
}

impl WorkHandler for ServiceBlockContext {
    fn handle_work(&self) {
        let work_dir = self.query_service(|service| service.definition.dir.clone());
        let block_status = self.get_block_status();
        let check_status = self.get_operation_status(OperationType::Check);
        let work_status = self.get_operation_status(OperationType::Work);

        let (step) = match block_status {
            BlockStatus::Working {
                step,
            } => step,
            _ => {
                error!("ERROR: invoked work-processing function with invalid block status {block_status:?}");
                return;
            }
        };

        let is_process = self.query_block(|block| match block.work {
            // Command sequences execute once and then are done
            WorkDefinition::CommandSeq { .. } => false,
            // Processes require that the work is in running-state in order for them to be healthy
            WorkDefinition::Process { .. } => true,
        });

        match step {
            // Ensure that there's no lingering process. There should not be if other actions are handled correctly,
            // but some defensive programming here doesn't hurt.
            WorkStep::Initial { skip_work_if_healthy } => {
                self.stop_all_operations_and_then(|| {
                    self.update_status(
                        BlockStatus::Working {
                            step: WorkStep::PrerequisiteCheck {
                                skip_work_if_healthy,
                                start_time: Instant::now(),
                                checks_completed: 0,
                                last_failure: None,
                            },
                        }
                    );
                });
            }

            WorkStep::PrerequisiteCheck {
                skip_work_if_healthy,
                start_time,
                checks_completed,
                last_failure,
            } => {
                let result = RequirementChecker {
                    all_requirements: self.query_block(|block| block.prerequisites.clone()),
                    current_requirement_idx: checks_completed,
                    timeout: None,
                    failure_wait_time: PRE_REQ_FAILURE_WAIT,
                    start_time,
                    last_failure,
                    context: &self,
                    workdir: self.query_service(|service| service.definition.dir.clone()),
                }.check_requirements();

                match result {
                    RequirementCheckResult::Working => {
                        // Do nothing intentionally, we're still processing
                    }
                    RequirementCheckResult::AllOk => {
                        self.update_status(
                            BlockStatus::Working {
                                step: if skip_work_if_healthy && !is_process {
                                    WorkStep::PreWorkHealthCheck {
                                        start_time: Instant::now(),
                                        checks_completed: 0,
                                    }
                                } else {
                                    WorkStep::PerformWork {
                                        steps_completed: 0
                                    }
                                },
                            },
                        );
                    }
                    RequirementCheckResult::CurrentCheckOk => {
                        self.update_status(
                            BlockStatus::Working {
                                step: WorkStep::PrerequisiteCheck {
                                    start_time,
                                    skip_work_if_healthy,
                                    checks_completed: checks_completed + 1,
                                    last_failure: None,
                                },
                            },
                        );
                    }
                    RequirementCheckResult::CurrentCheckFailed => {
                        self.update_status(
                            BlockStatus::Working {
                                step: WorkStep::PrerequisiteCheck {
                                    skip_work_if_healthy,
                                    start_time,
                                    checks_completed: 0,
                                    last_failure: Some(Instant::now()),
                                },
                            },
                        );
                    }
                    RequirementCheckResult::Timeout => {
                        error!("Prerequisite check timed out, even though timeout should not be possible");
                    }
                }
            }

            WorkStep::PreWorkHealthCheck { start_time, checks_completed } => {
                let result = RequirementChecker {
                    all_requirements: self.query_block(|block| block.health.requirements.clone()),
                    current_requirement_idx: checks_completed,
                    timeout: Some(Duration::from_secs(0)),
                    failure_wait_time: Duration::from_secs(0),
                    start_time,
                    last_failure: None,
                    context: &self,
                    workdir: self.query_service(|service| service.definition.dir.clone()),
                }.check_requirements();

                match result {
                    RequirementCheckResult::Working => {
                        // Do nothing intentionally, we're still processing
                    }
                    RequirementCheckResult::AllOk => {
                        // The block is healthy, we can move to OK status
                        self.update_status(BlockStatus::Ok);
                    }
                    RequirementCheckResult::CurrentCheckOk => {
                        // One check completed, move to check the next one
                        self.update_status(
                            BlockStatus::Working {
                                step: WorkStep::PreWorkHealthCheck {
                                    start_time,
                                    checks_completed: checks_completed + 1,
                                },
                            },
                        );
                    }
                    RequirementCheckResult::CurrentCheckFailed | RequirementCheckResult::Timeout => {
                        // If any check fails, then we must perform the work. Move to the appropriate state
                        self.update_status(
                            BlockStatus::Working {
                                step: WorkStep::PerformWork {
                                    steps_completed: 0,
                                },
                            },
                        );
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
                    WorkDefinition::Process { command: executable } => {
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
                let result = RequirementChecker {
                    all_requirements: self.query_block(|block| block.health.requirements.clone()),
                    current_requirement_idx: checks_completed,
                    timeout: Some(self.query_block(|block| block.health.timeout.clone())),
                    failure_wait_time: POST_WORK_HEALTH_FAILURE_WAIT,
                    start_time,
                    last_failure,
                    context: &self,
                    workdir: self.query_service(|service| service.definition.dir.clone()),
                }.check_requirements();

                match result {
                    // If the block is a process and we do not have a live process running, then immediately stop all
                    // work and enter error state
                    _ if is_process && !matches!(work_status, Some(AsyncOperationStatus::Running)) => {
                        self.stop_all_operations_and_then(|| {
                            self.add_ctrl_output("External process has terminated unexpectedly.".to_owned());
                            self.update_status(BlockStatus::Error)
                        });
                    }
                    // If there are no more (or at all) requirements to check, then we can finally consider the
                    // block healthy
                    RequirementCheckResult::AllOk => self.update_status(BlockStatus::Ok),
                    RequirementCheckResult::Timeout => self.update_status(BlockStatus::Error),
                    RequirementCheckResult::CurrentCheckOk => {
                        self.update_status(
                            BlockStatus::Working {
                                step: WorkStep::PostWorkHealthCheck {
                                    start_time,
                                    last_failure,
                                    checks_completed: checks_completed + 1,
                                },
                            },
                        );
                    }
                    RequirementCheckResult::CurrentCheckFailed => {
                        self.update_status(
                            BlockStatus::Working {
                                step: WorkStep::PostWorkHealthCheck {
                                    start_time,
                                    checks_completed: 0,
                                    last_failure: Some(Instant::now()),
                                },
                            },
                        );
                    }
                    RequirementCheckResult::Working => {
                        // Nothing to do, intentioanlly empty.
                    }
                }
            }
        }
    }
}

const PRE_REQ_FAILURE_WAIT: Duration = Duration::from_millis(500);
const POST_WORK_HEALTH_FAILURE_WAIT: Duration = Duration::from_millis(3000);