use std::process::Child;
use crate::runner::service_worker::{ConcurrentOperationStatus, WorkResult};
use crate::system_state::{OperationType};

pub trait WorkContext {
    fn stop_concurrent_operation(&self, operation_type: OperationType);
    
    fn clear_concurrent_operation(&self, operation_type: OperationType);
    
    fn stop_all_concurrent_operations(&self);
    
    fn get_concurrent_operation_status(&self, operation_type: OperationType) -> Option<ConcurrentOperationStatus>;

    fn perform_concurrent_work<F>(
        &self, work: F,
        operation_type: OperationType,
        silent: bool,
    ) where F: FnOnce() -> WorkResult + Send + 'static ;

    fn register_external_process(&self, handle: Child, operation_type: OperationType);

    fn create_rhai_scope(&self) -> rhai::Scope;

    fn add_system_output(&self, output: String);
}