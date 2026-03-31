use log::{error, info, warn};
use std::time::{Duration, Instant};

use crate::config::{Fingerprint, WorkDefinition};
use crate::models::{BlockAction, BlockStatus, WorkStep};
use crate::runner::fingerprint_checker::FingerprintChecker;
use crate::runner::service_worker::ConcurrentOperationStatus;
use crate::runner::service_worker::create_cmd::create_cmd;
use crate::runner::service_worker::requirement_checker::{
    RequirementCheckResult, RequirementChecker,
};
use crate::runner::service_worker::service_block_context::ServiceBlockContext;
use crate::runner::service_worker::work_sequence_executor::{
    WorkExecutionResult, WorkSequenceExecutor,
};
use crate::system_state::OperationType;
use crate::utils::format_err;

pub trait BlockProcessor {
    fn process_block(&self);
    fn handle_work(&self);
}
impl BlockProcessor for ServiceBlockContext {
    fn process_block(&self) {
        let debug_id = format!("{}.{}", self.service_id, self.block_id);
        let has_running_operations = [OperationType::Work, OperationType::Check].into_iter().any(
            |operation_type| match self.get_concurrent_operation_status(operation_type) {
                Some(ConcurrentOperationStatus::Running) => true,
                _ => false,
            },
        );

        match (self.get_block_status(), self.get_action()) {
            (
                BlockStatus::Disabled,
                Some(BlockAction::Enable) | Some(BlockAction::ToggleEnabled),
            ) => {
                self.clear_current_action();
                self.update_status(BlockStatus::Initial);
            }
            (_, Some(BlockAction::Enable)) => {
                self.clear_current_action();
            }

            (BlockStatus::Disabled, Some(BlockAction::Disable)) => {
                self.clear_current_action();
            }
            (_, Some(BlockAction::Disable)) if has_running_operations => {
                self.stop_all_operations();
            }
            (_, Some(BlockAction::Disable) | Some(BlockAction::ToggleEnabled)) => {
                self.clear_all_operations();
                self.clear_current_action();
                self.update_status(BlockStatus::Disabled);
            }

            (BlockStatus::Disabled, Some(_)) => {
                self.clear_current_action();
            }

            (BlockStatus::Working { .. }, None) => {
                self.handle_work();
            }

            (_, Some(BlockAction::ReRun)) if has_running_operations => {
                self.stop_all_operations();
            }
            (_, Some(BlockAction::ReRun)) => {
                self.clear_all_operations();
                self.clear_current_action();
                self.update_status(BlockStatus::Working {
                    step: WorkStep::initial(false),
                });
            }

            (BlockStatus::Initial | BlockStatus::Error, Some(BlockAction::Run)) => {
                info!("Block {debug_id} will be run");
                self.clear_current_action();

                self.update_status(BlockStatus::Working {
                    step: WorkStep::initial(true),
                });
            }

            (BlockStatus::Working { .. } | BlockStatus::Ok { .. }, Some(BlockAction::Run)) => {
                info!(
                    "Block {debug_id} is already in a running/finished status, clearing run-action"
                );
                self.clear_current_action();
            }
            (_, Some(BlockAction::Stop)) if has_running_operations => {
                self.stop_all_operations();
            }
            (status, Some(BlockAction::Stop)) => {
                self.clear_all_operations();
                self.clear_current_action();
                self.update_status(match status {
                    BlockStatus::Initial => BlockStatus::Initial,
                    BlockStatus::Working { .. } => BlockStatus::Initial,
                    BlockStatus::Ok { .. } => BlockStatus::Initial,
                    BlockStatus::Error => BlockStatus::Error,
                    BlockStatus::Disabled => BlockStatus::Disabled,
                })
            }

            (BlockStatus::Working { .. }, Some(BlockAction::Cancel)) if has_running_operations => {
                self.stop_all_operations();
            }
            (BlockStatus::Working { .. }, Some(BlockAction::Cancel)) => {
                self.clear_all_operations();
                self.clear_current_action();
            }

            (
                BlockStatus::Initial | BlockStatus::Ok { .. } | BlockStatus::Error,
                Some(BlockAction::Cancel),
            ) => {
                // Cancel should only stop the process if its in working-state. Stop is the action to use when wanting
                // to stop even OK-state blocks.
                self.clear_current_action();
            }

            (BlockStatus::Ok { .. }, None) => {
                let require_live_process = self.query_block(|block| match block.work {
                    WorkDefinition::CommandSeq { .. } => false,
                    WorkDefinition::Process { .. } => true,
                });

                match self.get_concurrent_operation_status(OperationType::Work) {
                    Some(ConcurrentOperationStatus::Running) => {
                        // Everything is OK
                    }
                    _ if require_live_process => {
                        // We don't have a live process and our work is of a type that it requires one. Likely the
                        // process has crashed or has been killed. Enter error-state
                        self.add_system_output(
                            "External process has terminated unexpectedly.".to_owned(),
                        );
                        self.update_status(BlockStatus::Error);
                    }
                    _ => {
                        // We don't have a live process but we don't require one either. Nothing to do.
                    }
                }
            }

            (_, None) => {
                // Intentionally do nothing: we're either currently performing some work, or are in some
                // other state with no action to execute
            }
        }
    }

    fn handle_work(&self) {
        let work_dir = self.query_service(|service| service.definition.workdir.clone());
        let block_status = self.get_block_status();
        let work_status = self.get_concurrent_operation_status(OperationType::Work);

        let step = match block_status {
            BlockStatus::Working { step } => step,
            _ => {
                error!(
                    "ERROR: invoked work-processing function with invalid block status {block_status:?}"
                );
                return;
            }
        };

        let is_process = self.query_block(|block| match block.work {
            // Command sequences execute once and then are done
            WorkDefinition::CommandSeq { .. } => false,
            // Processes require that the work is in running-state in order for them to be healthy
            WorkDefinition::Process { .. } => true,
        });

        let has_running_operations = [OperationType::Work, OperationType::Check].into_iter().any(
            |operation_type| match self.get_concurrent_operation_status(operation_type) {
                Some(ConcurrentOperationStatus::Running) => true,
                _ => false,
            },
        );

        match step {
            // Ensure that there's no lingering process. There should not be if other actions are handled correctly,
            // but some defensive programming here doesn't hurt.
            WorkStep::ResourceGroupCheck { .. } if has_running_operations => {
                self.stop_all_operations();
            }

            WorkStep::ResourceGroupCheck {
                skip_work_if_healthy,
            } => {
                self.clear_all_operations();
                let rg_in_use = if let Some(self_group) =
                    self.query_block(|block| block.resource_group.clone())
                {
                    self.query_state(|system_state| {
                        system_state
                            .current_profile
                            .as_ref()
                            .iter()
                            .flat_map(|profile| profile.services.iter())
                            .any(|service| {
                                // The resource group is in use if the profile has block that shares the same resource
                                // group, and is currently in working-state. But we do not count "resource group check"
                                // as a working state here.
                                service
                                    .definition
                                    .blocks
                                    .iter()
                                    .filter(|block| {
                                        block.resource_group.as_deref() == Some(self_group.as_str())
                                    })
                                    .any(|block| match service.get_block_status(&block.id) {
                                        BlockStatus::Working {
                                            step:
                                                WorkStep::ResourceGroupCheck { .. }
                                                | WorkStep::PrerequisiteCheck { .. },
                                        } => false,
                                        BlockStatus::Working { .. } => true,
                                        _ => false,
                                    })
                            })
                    })
                } else {
                    false
                };

                if !rg_in_use {
                    self.update_status(BlockStatus::Working {
                        step: if skip_work_if_healthy && !is_process {
                            WorkStep::PreWorkHealthCheck {
                                start_time: Instant::now(),
                                checks_completed: 0,
                            }
                        } else {
                            WorkStep::PerformWork {
                                current_step_started: Instant::now(),
                                steps_completed: 0,
                                new_fingerprint: None,
                            }
                        },
                    });
                }
            }

            WorkStep::PrerequisiteCheck {
                skip_work_if_healthy,
                start_time,
                checks_completed,
                last_failure,
            } => {
                let context = self.create_work_context(OperationType::Check, true);
                let result = RequirementChecker {
                    all_requirements: self.query_block(|block| block.prerequisites.clone()),
                    completed_count: checks_completed,
                    timeout: None,
                    failure_wait_time: PRE_REQ_FAILURE_WAIT,
                    start_time,
                    last_failure,
                    context: &context,
                    workdir: self.query_service(|service| service.definition.workdir.clone()),
                }
                .check_requirements();

                match result {
                    RequirementCheckResult::Working => {
                        // Do nothing intentionally, we're still processing
                    }
                    RequirementCheckResult::AllOk => {
                        self.update_status(BlockStatus::Working {
                            step: WorkStep::ResourceGroupCheck {
                                skip_work_if_healthy,
                            },
                        });
                    }
                    RequirementCheckResult::CurrentCheckOk => {
                        self.update_status(BlockStatus::Working {
                            step: WorkStep::PrerequisiteCheck {
                                start_time,
                                skip_work_if_healthy,
                                checks_completed: checks_completed + 1,
                                last_failure: None,
                            },
                        });
                    }
                    RequirementCheckResult::CurrentCheckFailed => {
                        self.update_status(BlockStatus::Working {
                            step: WorkStep::PrerequisiteCheck {
                                skip_work_if_healthy,
                                start_time,
                                checks_completed: 0,
                                last_failure: Some(Instant::now()),
                            },
                        });
                    }
                    RequirementCheckResult::Timeout => {
                        error!(
                            "Prerequisite check timed out, even though timeout should not be possible"
                        );
                    }
                }
            }

            WorkStep::PreWorkHealthCheck {
                start_time,
                checks_completed,
            } => {
                let context = self.create_work_context(OperationType::Check, false);
                let result = RequirementChecker {
                    all_requirements: self.query_block(|block| block.health.requirements.clone()),
                    completed_count: checks_completed,
                    timeout: Some(Duration::from_secs(0)),
                    failure_wait_time: Duration::from_secs(0),
                    start_time,
                    last_failure: None,
                    context: &context,
                    workdir: self.query_service(|service| service.definition.workdir.clone()),
                }
                .check_requirements();

                match result {
                    RequirementCheckResult::Working => {
                        // Do nothing intentionally, we're still processing
                    }
                    RequirementCheckResult::AllOk => {
                        // The block is healthy pre-work; check fingerprint before skipping work
                        self.update_status(BlockStatus::Working {
                            step: WorkStep::PreWorkFingerprintCheck,
                        });
                    }
                    RequirementCheckResult::CurrentCheckOk => {
                        // One check completed, move to check the next one
                        self.update_status(BlockStatus::Working {
                            step: WorkStep::PreWorkHealthCheck {
                                start_time,
                                checks_completed: checks_completed + 1,
                            },
                        });
                    }
                    RequirementCheckResult::CurrentCheckFailed
                    | RequirementCheckResult::Timeout => {
                        // If any check fails, then we must perform the work. Move to the appropriate state
                        self.update_status(BlockStatus::Working {
                            step: WorkStep::PerformWork {
                                current_step_started: Instant::now(),
                                steps_completed: 0,
                                new_fingerprint: None,
                            },
                        });
                    }
                }
            }

            WorkStep::PreWorkFingerprintCheck => {
                let fingerprint_def: Option<Fingerprint> =
                    self.query_block(|block| block.fingerprint.clone());

                match fingerprint_def {
                    None => {
                        // No fingerprint defined — treat as matching (skip work)
                        self.update_status(BlockStatus::Ok { was_worked: false });
                    }
                    Some(fingerprint) => {
                        let workdir =
                            self.query_service(|service| service.definition.workdir.clone());
                        let checker = FingerprintChecker::new(fingerprint, workdir);

                        match checker.calculate_checksum() {
                            Ok(current_fingerprint) => {
                                let stored = self.get_stored_fingerprint();

                                if stored.as_deref() == Some(current_fingerprint.as_str()) {
                                    // Fingerprints match — sources unchanged, skip work
                                    self.update_status(BlockStatus::Ok { was_worked: false });
                                } else {
                                    // Fingerprints differ — need to perform work
                                    self.add_system_output(format!(
                                        "Fingerprint changed (stored={}, current={}), performing work",
                                        stored.as_deref().unwrap_or("none"),
                                        &current_fingerprint
                                    ));
                                    self.update_status(BlockStatus::Working {
                                        step: WorkStep::PerformWork {
                                            current_step_started: Instant::now(),
                                            steps_completed: 0,
                                            new_fingerprint: Some(current_fingerprint),
                                        },
                                    });
                                }
                            }
                            Err(err) => {
                                warn!("Fingerprint calculation failed: {err}, performing work");
                                self.add_system_output(format!(
                                    "Fingerprint calculation failed: {err}, performing work"
                                ));
                                self.update_status(BlockStatus::Working {
                                    step: WorkStep::PerformWork {
                                        current_step_started: Instant::now(),
                                        steps_completed: 0,
                                        new_fingerprint: None,
                                    },
                                });
                            }
                        }
                    }
                }
            }

            WorkStep::PerformWork {
                steps_completed,
                current_step_started: step_started,
                new_fingerprint,
            } => {
                match self.query_block(|block| block.work.clone()) {
                    WorkDefinition::CommandSeq {
                        commands: executable_entries,
                    } => {
                        let context = self.create_work_context(OperationType::Work, false);
                        let result = WorkSequenceExecutor {
                            sequence: executable_entries
                                .iter()
                                .map(|entry| entry.clone().into())
                                .collect(),
                            completed_count: steps_completed,
                            entry_start_time: step_started,
                            last_recoverable_failure: None,
                            context: &context,
                            workdir: self
                                .query_service(|service| service.definition.workdir.clone()),
                        }
                        .exec_next();

                        match result {
                            // No recoverable failures here, go into error for any kind of issue
                            WorkExecutionResult::Failed
                            | WorkExecutionResult::RecoverableFailure => {
                                self.update_status(BlockStatus::Error);
                            }
                            WorkExecutionResult::EntryOk => {
                                self.update_status(BlockStatus::Working {
                                    step: WorkStep::PerformWork {
                                        current_step_started: Instant::now(),
                                        steps_completed: steps_completed + 1,
                                        new_fingerprint,
                                    },
                                })
                            }
                            WorkExecutionResult::AllOk => {
                                self.update_status(BlockStatus::Working {
                                    step: WorkStep::PostWorkHealthCheck {
                                        start_time: Instant::now(),
                                        checks_completed: 0,
                                        last_failure: None,
                                        new_fingerprint,
                                    },
                                })
                            }
                            WorkExecutionResult::Working => {
                                // Nothing to do but wait
                            }
                        }
                    }
                    WorkDefinition::Process {
                        command: executable,
                    } => {
                        match create_cmd(&executable, Some(work_dir)) {
                            Ok(mut command) => {
                                self.add_system_output(format!("Exec: {executable}"));

                                match command.spawn() {
                                    Ok(process_handle) => {
                                        // Process launched successfully, move to post-work health check
                                        self.register_external_process(
                                            process_handle,
                                            OperationType::Work,
                                        );
                                        self.update_status(BlockStatus::Working {
                                            step: WorkStep::PostWorkHealthCheck {
                                                start_time: Instant::now(),
                                                checks_completed: 0,
                                                last_failure: None,
                                                new_fingerprint: new_fingerprint.clone(),
                                            },
                                        })
                                    }
                                    Err(error) => {
                                        self.update_status(BlockStatus::Error);
                                        self.add_system_output(format_err!(
                                            "Failed to spawn child process",
                                            error
                                        ));
                                    }
                                }
                            }
                            Err(error) => {
                                self.update_status(BlockStatus::Error);
                                self.add_system_output(format_err!(
                                    "Error in command creation",
                                    error
                                ));
                            }
                        }
                    }
                }
            }

            WorkStep::PostWorkHealthCheck {
                start_time,
                checks_completed,
                last_failure,
                new_fingerprint,
            } => {
                let context = self.create_work_context(OperationType::Check, false);
                let result = RequirementChecker {
                    all_requirements: self.query_block(|block| block.health.requirements.clone()),
                    completed_count: checks_completed,
                    timeout: Some(self.query_block(|block| block.health.timeout)),
                    failure_wait_time: POST_WORK_HEALTH_FAILURE_WAIT,
                    start_time,
                    last_failure,
                    context: &context,
                    workdir: self.query_service(|service| service.definition.workdir.clone()),
                }
                .check_requirements();

                match result {
                    // If the block is a process and we do not have a live process running, then immediately stop all
                    // work and enter error state
                    _ if is_process
                        && !matches!(work_status, Some(ConcurrentOperationStatus::Running)) =>
                    {
                        self.clear_all_operations();
                        self.add_system_output(
                            "External process has terminated unexpectedly.".to_owned(),
                        );
                        self.update_status(BlockStatus::Error)
                    }
                    // If there are no more (or at all) requirements to check, then we can finally consider the
                    // block healthy
                    RequirementCheckResult::AllOk => {
                        if let Some(fp) = &new_fingerprint {
                            self.store_fingerprint(fp);
                        }
                        self.update_status(BlockStatus::Ok { was_worked: true })
                    }
                    RequirementCheckResult::Timeout => self.update_status(BlockStatus::Error),
                    RequirementCheckResult::CurrentCheckOk => {
                        self.update_status(BlockStatus::Working {
                            step: WorkStep::PostWorkHealthCheck {
                                start_time,
                                last_failure,
                                checks_completed: checks_completed + 1,
                                new_fingerprint,
                            },
                        });
                    }
                    RequirementCheckResult::CurrentCheckFailed => {
                        self.update_status(BlockStatus::Working {
                            step: WorkStep::PostWorkHealthCheck {
                                start_time,
                                checks_completed: 0,
                                last_failure: Some(Instant::now()),
                                new_fingerprint,
                            },
                        });
                    }
                    RequirementCheckResult::Working => {
                        // Nothing to do, empty on purpose.
                    }
                }
            }
        }
    }
}

const PRE_REQ_FAILURE_WAIT: Duration = Duration::from_millis(500);
const POST_WORK_HEALTH_FAILURE_WAIT: Duration = Duration::from_millis(3000);
