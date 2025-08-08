use crate::runner::service_worker::{ConcurrentOperationStatus, WorkResult};
use rhai::plugin::RhaiResult;
use std::process::Child;
use std::sync::mpsc::Receiver;

pub trait WorkContext {
    fn stop_concurrent_operation(&self);
    
    fn clear_concurrent_operation(&self);
    
    fn get_concurrent_operation_status(&self) -> Option<ConcurrentOperationStatus>;

    fn perform_concurrent_work<F>(
        &self, work: F,
    ) where F: FnOnce() -> WorkResult + Send + 'static ;

    fn register_external_process(&self, handle: Child);

    fn enqueue_rhai(&self, script: String, with_fn: bool) -> Receiver<RhaiResult>;

    fn add_system_output(&self, output: String);
}