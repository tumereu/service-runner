use std::process::Child;
use crate::runner::service_worker::{ConcurrentOperationStatus, WorkResult};

pub trait WorkContext {
    fn stop_concurrent_operation(&self);
    
    fn clear_concurrent_operation(&self);
    
    fn get_concurrent_operation_status(&self) -> Option<ConcurrentOperationStatus>;

    fn perform_concurrent_work<F>(
        &self, work: F,
    ) where F: FnOnce() -> WorkResult + Send + 'static ;

    fn register_external_process(&self, handle: Child);

    fn create_rhai_scope(&self) -> rhai::Scope;

    fn add_system_output(&self, output: String);
}