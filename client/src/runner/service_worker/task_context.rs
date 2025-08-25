use crate::config::TaskDefinitionId;
use crate::models::{OutputKey, OutputKind, Service, Task, TaskAction, TaskId, TaskStatus};
use crate::runner::scripting::executor::{ScriptExecutor, RhaiRequest};
use crate::runner::service_worker::work_context::WorkContext;
use crate::runner::service_worker::{
    ConcurrentOperationHandle, ConcurrentOperationStatus, ProcessWrapper, WorkResult,
    WorkWrapper,
};
use crate::system_state::{ConcurrentOperationKey, SystemState};
use log::{debug, error};
use rhai::plugin::RhaiResult;
use std::ops::Deref;
use std::process::Child;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, RwLock};

pub struct TaskContext {
    system_state: Arc<RwLock<SystemState>>,
    rhai_executor: Arc<ScriptExecutor>,
    pub task_id: TaskId,
}
impl TaskContext {
    pub fn new(
        system_state: Arc<RwLock<SystemState>>,
        rhai_executor: Arc<ScriptExecutor>,
        task_id: TaskId,
    ) -> Self {
        Self {
            system_state,
            rhai_executor,
            task_id,
        }
    }

    pub fn clear_current_action(&self) {
        let mut state = self.system_state.write().unwrap();
        state.update_task(&self.task_id, |task| {
            task.action = None;
        });
    }

    pub fn update_status(&self, status: TaskStatus) {
        let mut state = self.system_state.write().unwrap();
        state.update_task(&self.task_id, |task| {
            task.status = status;
        });
    }

    pub fn update_task<F>(&mut self, update: F)
    where
        F: for<'a> FnOnce(&'a mut Task),
    {
        let mut state = self.system_state.write().unwrap();
        state.update_task(&self.task_id, update);
    }

    pub fn query_service<R, F>(&self, query: F) -> Option<R>
    where
        F: for<'a> FnOnce(&'a Service) -> R,
        R: 'static,
    {
        let service_id = self.query_task(|task| task.service_id.clone())?;
        let state = self.system_state.read().unwrap();
        Some(query(state.get_service(&service_id)?))
    }

    pub fn query_system<R, F>(&self, query: F) -> R
    where
        F: for<'a> FnOnce(&'a SystemState) -> R,
        R: 'static,
    {
        let state = self.system_state.read().unwrap();
        query(&state)
    }

    pub fn update_system<R, F>(&self, query: F)
    where
        F: for<'a> FnOnce(&'a mut SystemState),
    {
        let mut state = self.system_state.write().unwrap();
        query(&mut state);
    }

    pub fn query_task<R, F>(&self, query: F) -> R
    where
        F: FnOnce(&Task) -> R,
        R: 'static,
    {
        let state = self.system_state.read().unwrap();
        let task = state.get_task(&self.task_id).unwrap();

        query(task)
    }

    pub fn get_action(&self) -> Option<TaskAction> {
        self.query_task(|task| task.action.clone())
    }

    pub fn get_status(&self) -> TaskStatus {
        self.query_task(|task| task.status.clone())
    }

    pub fn get_task_definition_id(&self) -> TaskDefinitionId {
        self.query_system(|system| {
            system.get_task(&self.task_id).unwrap().definition.id.clone()
        })
    }

    pub fn stop_concurrent_operation(&self) {
        self.system_state
            .write()
            .unwrap()
            .get_concurrent_operation(&ConcurrentOperationKey::Task {
                task_id: self.task_id.clone(),
            })
            .iter()
            .for_each(|operation| operation.stop());
    }

    pub fn clear_concurrent_operation(&self) {
        match self.get_concurrent_operation_status() {
            Some(ConcurrentOperationStatus::Running) => {
                error!(
                    "Received request to clear stopped operation for task {id} but operation is still running",
                    id = self.task_id,
                )
            }
            Some(ConcurrentOperationStatus::Failed | ConcurrentOperationStatus::Ok) => {
                debug!(
                    "Removing stopped operation of type for {id}",
                    id = self.task_id,
                );

                self.system_state.write().unwrap().set_concurrent_operation(
                    ConcurrentOperationKey::Task {
                        task_id: self.task_id.clone(),
                    },
                    None,
                );
            }
            None => {
                // No need to do anything, no operation to remove
            }
        }
    }

    pub fn get_concurrent_operation_status(&self) -> Option<ConcurrentOperationStatus> {
        self.system_state
            .read()
            .unwrap()
            .get_concurrent_operation(&ConcurrentOperationKey::Task {
                task_id: self.task_id.clone(),
            })
            .map(|operation| operation.status())
    }

    pub fn create_work_context(&self, silent: bool) -> TaskWorkContext {
        TaskWorkContext {
            task_context: self,
            silent,
        }
    }
}

pub struct TaskWorkContext<'a> {
    task_context: &'a TaskContext,
    silent: bool
}
impl<'a> Deref for TaskWorkContext<'a> {
    type Target = &'a TaskContext;

    fn deref(&self) -> &Self::Target {
        &self.task_context
    }
}

impl WorkContext for TaskWorkContext<'_> {
    fn stop_concurrent_operation(&self) {
        self.task_context.stop_concurrent_operation();
    }

    fn clear_concurrent_operation(&self) {
        self.task_context.clear_concurrent_operation();
    }

    fn get_concurrent_operation_status(&self) -> Option<ConcurrentOperationStatus> {
        self.task_context.get_concurrent_operation_status()
    }

    fn perform_concurrent_work<F>(&self, work: F)
    where
        F: FnOnce() -> WorkResult + Send + 'static,
    {
        let wrapper = WorkWrapper::wrap(
            self.system_state.clone(),
            self.query_task(|task| task.service_id.clone()),
            self.get_task_definition_id().0,
            self.silent,
            work,
        );
        self.system_state.write().unwrap().set_concurrent_operation(
            ConcurrentOperationKey::Task {
                task_id: self.task_id.clone(),
            },
            Some(ConcurrentOperationHandle::Work(wrapper)),
        );
    }

    fn register_external_process(&self, handle: Child) {
        let wrapper = ProcessWrapper::wrap(
            self.system_state.clone(),
            self.query_task(|task| task.service_id.clone()),
            self.get_task_definition_id().0,
            handle,
        );

        self.system_state.write().unwrap().set_concurrent_operation(
            ConcurrentOperationKey::Task {
                task_id: self.task_id.clone(),
            },
            Some(ConcurrentOperationHandle::Process(wrapper)),
        );
    }

    fn enqueue_rhai(&self, script: String, allow_fn: bool) -> Receiver<RhaiResult> {
        let service_id = self.query_task(|task| task.service_id.clone());
        
        self.rhai_executor.enqueue(RhaiRequest {
            script,
            allow_functions: allow_fn,
            service_id,
        })
    }
    
    fn add_system_output(&self, output: String) {
        let service_id = self.query_task(|task| task.service_id.clone());
        let source_name = self.get_task_definition_id().0;

        self.system_state
            .write()
            .unwrap()
            .add_output(
                &OutputKey {
                    service_id,
                    source_name,
                    kind: OutputKind::System
                },
                output
            );
    }
}
