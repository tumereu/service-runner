use crate::models::{BlockAction, BlockStatus, GetBlock, Service};
use crate::system_state::SystemState;
use std::sync::{Arc, Mutex};
use log::{debug, error};
use crate::config::Block;
use crate::runner::service_worker::{AsyncOperationStatus, CtrlOutputWriter, WorkWrapper};

pub struct BlockWorker {
    system_state: Arc<Mutex<SystemState>>,
    pub service_id: String,
    pub block_id: String,
}
impl BlockWorker {
    pub fn new(system_state: Arc<Mutex<SystemState>>, service_id: String, block_id: String) -> Self {
        Self {
            system_state,
            service_id,
            block_id,
        }
    }

    pub fn clear_current_action(&self) {
        let mut state = self.system_state.lock().unwrap();
        state.update_service(&self.service_id, |service| {
            service.update_block_action(&self.block_id, None)
        })
    }

    pub fn update_status(&self, status: BlockStatus) {
        let mut state = self.system_state.lock().unwrap();
        state.update_service(&self.service_id, |service| {
            service.update_block_status(&self.block_id, status)
        });
    }

    pub fn update_service<F>(&self, update: F) where F : FnOnce(&mut Service) {
        self.system_state.lock().unwrap().update_service(&self.service_id, update);
    }

    pub fn query_system<R, F>(&self, query: F) -> R where F : FnOnce(&SystemState) -> R {
        let state = self.system_state.lock().unwrap();

        query(&state)
    }

    pub fn query_service<R, F>(&self, query: F) -> R where F : FnOnce(&Service) -> R {
        let service = self.system_state.lock().unwrap()
            .get_service(&self.service_id)
            .unwrap();

        query(service)
    }

    pub fn query_block<R, F>(&self, query: F) -> R where F : FnOnce(&Block) -> R {
        let block = self.system_state.lock().unwrap()
            .get_service(&self.service_id)
            .unwrap()
            .get_block(&self.block_id)
            .unwrap();

        query(block)
    }

    pub fn get_action(&self) -> BlockAction {
        self.query_service(|service| service.get_block_action(&self.block_id).unwrap().clone())
    }

    pub fn get_block_status(&self) -> BlockStatus {
        self.query_service(|service| service.get_block_status(&self.block_id).unwrap().clone())
    }

    pub fn get_operation_status(&self) -> Option<AsyncOperationStatus> {
        self.system_state
            .lock()
            .unwrap()
            .get_block_operation(&self.service_id, &self.block_id)
            .map(|operation| operation.status())
    }

    /// A call to this will do one (and only one of the following) of the following.
    /// - Issue a stop signal to the current operation of this block, if it is running, or
    /// - remove the current block operation from system state if it exists and is stopped, or
    /// - executes the given function if the block has no stored operation.
    pub fn stop_operation_and_then<F>(
        &self,
        execute: F,
    ) where
        F: FnOnce(),
    {
        let debug_id = format!("{}.{}", self.service_id, self.block_id);

        match self.get_operation_status() {
            Some(AsyncOperationStatus::Running) => {
                debug!("Stopping current operation for {debug_id}");
                self.system_state
                    .lock()
                    .unwrap()
                    .get_block_operation(&self.service_id, &self.block_id)
                    .iter()
                    .for_each(|operation| operation.stop());
            }
            Some(status) => {
                debug!("Current operation for {debug_id} has stopped ({status:?}), removing it");

                self.system_state
                    .lock()
                    .unwrap()
                    .set_block_operation(&self.service_id, &self.block_id, None)
            }
            None => {
                execute();
            }
        }
    }

    pub fn clear_stopped_operation(&self) {
        let debug_id = format!("{}.{}", self.service_id, self.block_id);

        match self.get_operation_status() {
            Some(AsyncOperationStatus::Running) => {
                error!("Received request to clear stopped operation for {debug_id} but operation is still running")
            }
            Some(AsyncOperationStatus::Failed | AsyncOperationStatus::Ok) => {
                debug!("Removing stopped operation for {debug_id}");

                self.system_state
                    .lock()
                    .unwrap()
                    .set_block_operation(&self.service_id, &self.block_id, None);
            }
            None => {
                // No need to do anything, no operation to remove
            }
        }
    }

    pub fn perform_async_work<F>(&self, work: F) where F : FnOnce() -> bool {
        WorkWrapper::wrap(
            self.system_state.clone(),
            self.service_id.clone(),
            self.block_id.clone(),
            work,
        );
    }

    pub fn add_ctrl_output(&self, output: String) {
        self.system_state.lock().unwrap().add_ctrl_output(
            &self.service_id,
            output
        );
    }
}
