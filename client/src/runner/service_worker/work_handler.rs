use std::net::TcpListener;
use crate::config::{HttpMethod, RequiredStatus, Requirement, WorkDefinition};
use crate::models::{BlockStatus, GetBlock, OutputKey, OutputKind, WorkStep};
use crate::runner::service_worker::process_wrapper::{create_cmd, ProcessWrapper};
use crate::runner::service_worker::{
    AsyncOperationHandle, AsyncOperationStatus, CtrlOutputWriter, WorkWrapper,
};
use crate::system_state::SystemState;
use crate::utils::format_err;
use log::{debug, error};
use reqwest::blocking::Client as HttpClient;
use reqwest::Method;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crate::models::WorkStep::Initial;
use crate::runner::service_worker::block_worker::BlockWorker;
use crate::runner::service_worker::req_checker::RequirementChecker;

pub trait WorkHandler {
    fn handle_work(&self);
}

impl WorkHandler for BlockWorker {
    fn handle_work(&self) {
        let block_status = self.get_block_status();
        let operation_status = self.get_operation_status();
        let work_dir = self.query_service(|service| service.definition.dir.clone());

        let (step, skip_if_healthy) = match block_status {
            BlockStatus::Working {
                step,
                skip_if_healthy,
            } => (step, skip_if_healthy),
            _ => {
                error!("ERROR: invoked work-processing function with invalid block status {status:?}");
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
                self.stop_operation_and_then(|| {
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
                let current_requirement = self.query_block(|block| block.prerequisites.get(checks_completed).clone());

                match (operation_status, current_requirement) {
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
                        self.perform_async_work(|| self.check_requirement(&requirement));
                    }
                    (Some(AsyncOperationStatus::Failed), _) => {
                        self.clear_stopped_operation();
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
                        self.clear_stopped_operation();
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
                    block.health.requirements.get(checks_completed).clone()
                });
                let has_health_checks = self.query_block(|block| !block.health.requirements.is_empty());

                match (operation_status, current_requirement) {
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
                        self.perform_async_work(|| self.check_requirement(&requirement));
                    }
                    // Health check failed, we must fully perform the block's work. Move into the appropriate state
                    (Some(AsyncOperationStatus::Failed), _) => {
                        self.clear_stopped_operation();
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
                        self.clear_stopped_operation();
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

            WorkStep::PerformWork { steps_completed } => {
                match self.query_block(|block| block.work.clone()) {
                    WorkDefinition::CommandSeq { commands } => {
                        let next_command = &commands[steps_completed];
                        let mut command = create_cmd(
                            next_command,
                            Some(work_dir),
                        );

                        self.add_ctrl_output(
                            format!("Exec: {next_command}"),
                        );

                        match command.spawn() {
                            Ok(process_handle) => {
                                let wrapper = ProcessWrapper::wrap(
                                    state_arc.clone(),
                                    process_handle,
                                    service_id.to_owned(),
                                    block_id.to_owned(),
                                );

                                let mut state = state_arc.lock().unwrap();
                                state.set_block_operation(
                                    service_id,
                                    block_id,
                                    Some(AsyncOperationHandle::Process(wrapper)),
                                );
                            }
                            Err(error) => {
                                let mut state = state_arc.lock().unwrap();
                                state.update_service(&service_id, |service| {
                                    service.update_block_status(&block_id, BlockStatus::Error)
                                });

                                state.add_output(
                                    &OutputKey {
                                        name: OutputKey::CTL.into(),
                                        service_ref: service_id.to_owned(),
                                        kind: OutputKind::Compile,
                                    },
                                    format_err!("Failed to spawn child process", error),
                                );
                            }
                        }
                    }
                    WorkDefinition::Process { executable } => {
                        // TODO handle
                    }
                }
            }
        }
    }
}

const PRE_REQ_FAILURE_WAIT: Duration = Duration::from_millis(500);