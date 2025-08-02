use log::info;
use crate::models::{BlockAction, BlockStatus, WorkStep};
use crate::runner::service_worker::block_worker::BlockWorker;
use crate::runner::service_worker::work_handler;

pub trait BlockProcessor {
    fn process_block(&self);
}
impl BlockProcessor for BlockWorker {
    fn process_block(&self) {
        let service_enabled = self.query_service(|service| service.enabled);

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
                self.stop_block_operation_and_then(|| {
                    self.clear_current_action();
                    self.update_service(|service| service.enabled = false);
                });
            }
            (_, Some(BlockAction::ToggleEnabled)) => {
                self.stop_block_operation_and_then(|| {
                    self.clear_current_action();
                    self.update_service(|service| service.enabled = false);
                });
            }

            (_, Some(_)) if !service_enabled => {
                self.clear_current_action();
            }

            (BlockStatus::Working { .. }, None) => {
                work_handler::handle_work(state_arc.clone(), service_id, block_id);
            }

            (_, Some(BlockAction::ReRun)) => {
                work_handler::stop_block_operation_and_then(state_arc.clone(), service_id, block_id, || {
                    self.clear_current_action();
                    update_status(
                        state_arc.clone(),
                        service_id,
                        block_id,
                        BlockStatus::Working {
                            skip_if_healthy: false,
                            step: WorkStep::default(),
                        },
                    )
                });
            }

            (BlockStatus::Initial | BlockStatus::Error, Some(BlockAction::Run)) => {
                info!("Block {service_id}.{block_id} will be run");
                self.clear_current_action();

                update_status(
                    state_arc.clone(),
                    service_id,
                    block_id,
                    BlockStatus::Working {
                        skip_if_healthy: true,
                        step: WorkStep::default(),
                    },
                )
            }

            (BlockStatus::Working { .. } | BlockStatus::Ok, Some(BlockAction::Run)) => {
                info!("Block {service_id}.{block_id} is already in a running/finished status, clearing run-action");
                self.clear_current_action();
            }
            (status, Some(BlockAction::Stop)) => {
                work_handler::stop_block_operation_and_then(state_arc.clone(), service_id, block_id, || {
                    self.clear_current_action();
                    update_status(
                        state_arc.clone(),
                        service_id,
                        block_id,
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
                work_handler::stop_block_operation_and_then(state_arc.clone(), service_id, block_id, || {
                    self.clear_current_action();
                });
            }
            (
                BlockStatus::Initial | BlockStatus::Ok | BlockStatus::Error,
                Some(BlockAction::Cancel),
            ) => {
                self.clear_current_action();
            }

            (_, None) => {
                // Intentionally do nothing: we're either currently performing some work, or are in some
                // other state with no action to execute
            }
        }
    }
}
