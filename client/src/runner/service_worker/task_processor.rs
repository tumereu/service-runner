use crate::runner::service_worker::task_context::TaskContext;
use crate::runner::service_worker::work_context::WorkContext;

pub trait TaskProcessor {
    fn process_task(&self);
}
impl TaskProcessor for TaskContext {
    fn process_task(&self) {
        /*
        let task_status = self.get_status();
        let operation_status = self.get_concurrent_operation_status(O)

        match self.get_status() {

        }
        FIXME
         */
    }
}
