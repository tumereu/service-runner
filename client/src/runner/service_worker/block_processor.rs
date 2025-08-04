use log::info;

use crate::config::WorkDefinition;
use crate::models::{BlockAction, BlockStatus, WorkStep};
use crate::runner::service_worker::ConcurrentOperationStatus;
use crate::runner::service_worker::service_block_context::ServiceBlockContext;
use crate::runner::service_worker::work_context::WorkContext;
use crate::runner::service_worker::work_handler::WorkHandler;
use crate::system_state::OperationType;

pub trait BlockProcessor {
    fn process_block(&self);
}
impl BlockProcessor for ServiceBlockContext {
    fn process_block(&self) {
        let service_enabled = self.query_service(|service| service.enabled);
        let debug_id = format!("{}.{}", self.service_id, self.block_id);

        match (self.get_block_status(), self.get_action()) {
            (_, Some(BlockAction::Enable)) => {
                self.clear_current_action();
                self.update_service(|service| service.enabled = true);
            }
            (_, Some(BlockAction::ToggleEnabled)) if !service_enabled => {
                self.clear_current_action();
                self.update_service(|service| service.enabled = true);
            }

            (_, Some(BlockAction::Disable)) => {
                self.stop_all_operations_and_then(|| {
                    self.clear_current_action();
                    self.update_service(|service| service.enabled = false);
                });
            }
            (_, Some(BlockAction::ToggleEnabled)) => {
                self.stop_all_operations_and_then(|| {
                    self.clear_current_action();
                    self.update_service(|service| service.enabled = false);
                });
            }

            (_, Some(_)) if !service_enabled => {
                self.clear_current_action();
            }

            (BlockStatus::Working { .. }, None) => {
                self.handle_work();
            }

            (_, Some(BlockAction::ReRun)) => {
                self.stop_all_operations_and_then(|| {
                    info!("Re-running {debug_id}");

                    self.clear_current_action();
                    self.update_status(
                        BlockStatus::Working {
                            step: WorkStep::Initial { skip_work_if_healthy: false },
                        },
                    );
                });
            }

            (BlockStatus::Initial | BlockStatus::Error, Some(BlockAction::Run)) => {
                info!("Block {debug_id} will be run");
                self.clear_current_action();

                self.update_status(
                    BlockStatus::Working {
                        step: WorkStep::Initial { skip_work_if_healthy: true },
                    },
                );
            }

            (BlockStatus::Working { .. } | BlockStatus::Ok, Some(BlockAction::Run)) => {
                info!("Block {debug_id} is already in a running/finished status, clearing run-action");
                self.clear_current_action();
            }
            (status, Some(BlockAction::Stop)) => {
                self.stop_all_operations_and_then(|| {
                    self.clear_current_action();
                    self.update_status(
                        match status {
                            BlockStatus::Initial => BlockStatus::Initial,
                            BlockStatus::Working { .. } => BlockStatus::Initial,
                            // FIXME maybe this should be based on work type, or maybe health check?
                            //       in any case, we can't always go back to initial. Or can we?
                            BlockStatus::Ok => BlockStatus::Initial,
                            BlockStatus::Error => BlockStatus::Error,
                        },
                    )
                });
            }
            (BlockStatus::Working { .. }, Some(BlockAction::Cancel)) => {
                self.stop_all_operations_and_then(|| {
                    self.clear_current_action();
                });
            }
            (
                BlockStatus::Initial | BlockStatus::Ok | BlockStatus::Error,
                Some(BlockAction::Cancel),
            ) => {
                // Cancel should only stop the process if its in working-state. Stop is the action to use when wanting
                // to stop even OK-state blocks.
                self.clear_current_action();
            }

            (BlockStatus::Ok, None) => {
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
                        self.add_ctrl_output("External process has terminated unexpectedly.".to_owned());
                        self.update_status(BlockStatus::Error);
                    }
                    _ => {
                        // We don't have a live process but we don't require one either. Nothing to do.
                    }
                }
            },

            (_, None) => {
                // Intentionally do nothing: we're either currently performing some work, or are in some
                // other state with no action to execute
            }
        }
    }
}
