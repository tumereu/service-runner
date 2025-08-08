use crate::config::Block;
use crate::models::{BlockAction, BlockStatus, GetBlock, OutputKey, OutputKind, Service};
use crate::runner::rhai::{RhaiExecutor, RhaiRequest};
use crate::runner::service_worker::work_context::WorkContext;
use crate::runner::service_worker::{
    ConcurrentOperationHandle, ConcurrentOperationStatus, ProcessWrapper, WorkResult,
    WorkWrapper,
};
use crate::system_state::{ConcurrentOperationKey, OperationType, SystemState};
use log::{debug, error};
use rhai::plugin::RhaiResult;
use std::ops::Deref;
use std::process::Child;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

pub struct ServiceBlockContext {
    system_state: Arc<Mutex<SystemState>>,
    rhai_executor: Arc<RhaiExecutor>,
    pub service_id: String,
    pub block_id: String,
}
impl ServiceBlockContext {
    pub fn new(
        system_state: Arc<Mutex<SystemState>>,
        rhai_executor: Arc<RhaiExecutor>,
        service_id: String,
        block_id: String,
    ) -> Self {
        Self {
            system_state,
            rhai_executor,
            service_id,
            block_id,
        }
    }

    pub fn query_state< R, F>(&self, query: F) -> R
    where
        F: for<'a> FnOnce(&'a SystemState) -> R,
        R: 'static,
    {
        let state = self.system_state.lock().unwrap();
        let result = query(&*state);
        result
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

    pub fn update_service<F>(&self, update: F)
    where
        F: for<'a> FnOnce(&'a mut Service),
    {
        self.system_state
            .lock()
            .unwrap()
            .update_service(&self.service_id, update);
    }

    pub fn query_service<R, F>(&self, query: F) -> R
    where
        F: for<'a> FnOnce(&'a Service) -> R,
        R: 'static,
    {
        let state = self.system_state.lock().unwrap();
        let service = state.get_service(&self.service_id).unwrap();

        query(service)
    }

    pub fn query_block<R, F>(&self, query: F) -> R
    where
        F: for<'a> FnOnce(&'a Block) -> R,
        R: 'static,
    {
        let state = self.system_state.lock().unwrap();
        let block = state
            .get_service(&self.service_id)
            .unwrap()
            .get_block(&self.block_id)
            .unwrap();

        query(block)
    }

    pub fn get_action(&self) -> Option<BlockAction> {
        self.query_service(|service| service.get_block_action(&self.block_id))
    }

    pub fn get_block_status(&self) -> BlockStatus {
        self.query_service(|service| service.get_block_status(&self.block_id))
    }

    pub fn get_concurrent_operation_status(&self, operation_type: OperationType) -> Option<ConcurrentOperationStatus> {
        self.system_state
            .lock()
            .unwrap()
            .get_concurrent_operation(&ConcurrentOperationKey::Block {
                service_id: self.service_id.clone(),
                block_id: self.block_id.clone(),
                operation_type,
            }).map(|operation| operation.status())
    }

    pub fn stop_concurrent_operation(&self, operation_type: OperationType) {
        self.system_state
            .lock()
            .unwrap()
            .get_concurrent_operation(&ConcurrentOperationKey::Block {
                service_id: self.service_id.clone(),
                block_id: self.block_id.clone(),
                operation_type,
            })
            .iter()
            .for_each(|operation| operation.stop());
    }

    pub fn stop_all_operations(&self) {
        self.stop_concurrent_operation(OperationType::Check);
        self.stop_concurrent_operation(OperationType::Work);
    }

    pub fn clear_all_operations(&self) {
        [
            OperationType::Check,
            OperationType::Work,
        ].into_iter().for_each(|operation_type| {
            let debug_id = format!("{}.{}", self.service_id, self.block_id);

            match self.get_concurrent_operation_status(operation_type) {
                Some(ConcurrentOperationStatus::Running) => {
                    error!("Received request to clear stopped operation for {debug_id} but operation is still running")
                }
                Some(ConcurrentOperationStatus::Failed | ConcurrentOperationStatus::Ok) => {
                    debug!("Removing stopped operation for {debug_id}");

                    self.system_state.lock().unwrap().set_concurrent_operation(
                        ConcurrentOperationKey::Block {
                            service_id: self.service_id.clone(),
                            block_id: self.block_id.clone(),
                            operation_type,
                        },
                        None,
                    );
                }
                None => {
                    // No need to do anything, no operation to remove
                }
            }
        });
    }

    pub fn create_work_context(&self, operation_type: OperationType, silent: bool) -> BlockWorkContext {
        BlockWorkContext {
            block_context: self,
            operation_type,
            silent,
        }
    }

    pub fn add_system_output(&self, output: String) {
        self.system_state
            .lock()
            .unwrap()
            .add_output(
                &OutputKey {
                    service_id: Some(self.service_id.clone()),
                    source_name: self.block_id.clone(),
                    kind: OutputKind::System
                },
                output
            );
    }

    pub fn register_external_process(&self, handle: Child, operation_type: OperationType) {
        let wrapper = ProcessWrapper::wrap(
            self.system_state.clone(),
            Some(self.service_id.clone()),
            self.block_id.clone(),
            handle,
        );

        self.system_state.lock().unwrap().set_concurrent_operation(
            ConcurrentOperationKey::Block {
                service_id: self.service_id.clone(),
                block_id: self.block_id.clone(),
                operation_type: operation_type.clone(),
            },
            Some(ConcurrentOperationHandle::Process(wrapper)),
        );
    }
}

pub struct BlockWorkContext<'a> {
    block_context: &'a ServiceBlockContext,
    operation_type: OperationType,
    silent: bool
}
impl<'a> Deref for BlockWorkContext<'a> {
    type Target = &'a ServiceBlockContext;

    fn deref(&self) -> &Self::Target {
        &self.block_context
    }
}

impl WorkContext for BlockWorkContext<'_> {
    fn stop_concurrent_operation(&self) {
        self.block_context.stop_concurrent_operation(self.operation_type);
    }

    fn clear_concurrent_operation(&self) {
        let debug_id = format!("{}.{}", self.service_id, self.block_id);

        match self.get_concurrent_operation_status() {
            Some(ConcurrentOperationStatus::Running) => {
                error!("Received request to clear stopped operation for {debug_id} but operation is still running")
            }
            Some(ConcurrentOperationStatus::Failed | ConcurrentOperationStatus::Ok) => {
                debug!("Removing stopped operation for {debug_id}");

                self.system_state.lock().unwrap().set_concurrent_operation(
                    ConcurrentOperationKey::Block {
                        service_id: self.service_id.clone(),
                        block_id: self.block_id.clone(),
                        operation_type: self.operation_type.clone(),
                    },
                    None,
                );
            }
            None => {
                // No need to do anything, no operation to remove
            }
        }
    }

    fn get_concurrent_operation_status(&self) -> Option<ConcurrentOperationStatus> {
        self.block_context.get_concurrent_operation_status(self.operation_type)
    }

    fn perform_concurrent_work<F>(&self, work: F)
    where
        F: FnOnce() -> WorkResult + Send + 'static,
    {
        let wrapper = WorkWrapper::wrap(
            self.system_state.clone(),
            Some(self.service_id.clone()),
            self.block_id.clone(),
            self.silent,
            work,
        );
        self.system_state.lock().unwrap().set_concurrent_operation(
            ConcurrentOperationKey::Block {
                service_id: self.service_id.clone(),
                block_id: self.block_id.clone(),
                operation_type: self.operation_type.clone(),
            },
            Some(ConcurrentOperationHandle::Work(wrapper)),
        );
    }

    fn register_external_process(&self, handle: Child) {
        self.block_context.register_external_process(handle, self.operation_type);
    }

    fn enqueue_rhai(&self, script: String, allow_fn: bool) -> Receiver<RhaiResult> {
        self.rhai_executor.enqueue(RhaiRequest {
            script,
            allow_functions: allow_fn,
            service_id: Some(self.service_id.clone())
        })
    }

    fn add_system_output(&self, output: String) {
        self.block_context.add_system_output(output);
    }
}
