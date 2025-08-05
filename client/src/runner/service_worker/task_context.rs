use std::process::Child;
use std::sync::{Arc, Mutex};

use log::{debug, error};

use crate::config::{Block, TaskDefinitionId};
use crate::models::{BlockAction, BlockStatus, GetBlock, OutputKey, OutputKind, Service, Task, TaskAction, TaskId, TaskStatus};
use crate::rhai::populate_rhai_scope;
use crate::runner::service_worker::work_context::WorkContext;
use crate::runner::service_worker::{
    ConcurrentOperationHandle, ConcurrentOperationStatus, ProcessWrapper, WorkResult,
    WorkWrapper,
};
use crate::system_state::{ConcurrentOperationKey, OperationType, SystemState};

pub struct TaskContext {
    system_state: Arc<Mutex<SystemState>>,
    pub task_id: TaskId,
    pub definition_id: TaskDefinitionId,
}
impl TaskContext {
    pub fn new(
        system_state: Arc<Mutex<SystemState>>,
        task_id: TaskId,
        definition_id: TaskDefinitionId,
    ) -> Self {
        Self {
            system_state,
            task_id,
            definition_id,
        }
    }

    pub fn clear_current_action(&self) {
        let mut state = self.system_state.lock().unwrap();
        state.update_task(&self.task_id, |task| {
            task.action = None;
        });
    }

    pub fn update_status(&self, status: TaskStatus) {
        let mut state = self.system_state.lock().unwrap();
        state.update_task(&self.task_id, |task| {
            task.status = status;
        });
    }

    pub fn update_task<F>(&mut self, update: F)
    where
        F: for<'a> FnOnce(&'a mut Task),
    {
        let mut state = self.system_state.lock().unwrap();
        state.update_task(&self.task_id, update);
    }

    pub fn query_system<R, F>(&self, query: F) -> R
    where
        F: for<'a> FnOnce(&'a SystemState) -> R,
        R: 'static,
    {
        let state = self.system_state.lock().unwrap();

        let result = query(&state);
        drop(state);

        result
    }

    pub fn update_system<R, F>(&self, query: F)
    where
        F: for<'a> FnOnce(&'a mut SystemState),
    {
        let mut state = self.system_state.lock().unwrap();

        query(&mut state);
    }

    pub fn query_task<R, F>(&self, query: F) -> R
    where
        F: FnOnce(&Task) -> R,
        R: 'static,
    {
        let state = self.system_state.lock().unwrap();
        let task = state.get_task(&self.task_id).unwrap();

        query(task)
    }

    pub fn get_action(&self) -> Option<TaskAction> {
        self.query_task(|task| task.action.clone())
    }

    pub fn get_status(&self) -> TaskStatus {
        self.query_task(|task| task.status.clone())
    }

    fn get_task_definition_id(&self) -> TaskDefinitionId {
        self.query_system(|system| {
            system.get_task(&self.task_id).unwrap().definition_id.clone()
        })
    }
}

impl WorkContext for &TaskContext {
    fn stop_concurrent_operation(&self, operation_type: OperationType) {
        self.system_state
            .lock()
            .unwrap()
            .get_concurrent_operation(&ConcurrentOperationKey::Task {
                task_id: self.task_id.clone(),
                operation_type,
            })
            .iter()
            .for_each(|operation| operation.stop());
    }

    fn clear_concurrent_operation(&self, operation_type: OperationType) {
        match self.get_concurrent_operation_status(operation_type.clone()) {
            Some(ConcurrentOperationStatus::Running) => {
                error!(
                    "Received request to clear stopped operation {operation_type:?} for task {id} but operation is still running",
                    operation_type = operation_type,
                    id = self.task_id,
                )
            }
            Some(ConcurrentOperationStatus::Failed | ConcurrentOperationStatus::Ok) => {
                debug!(
                    "Removing stopped operation of type {operation_type:?} for {id}",
                    operation_type = operation_type,
                    id = self.task_id,
                );

                self.system_state.lock().unwrap().set_concurrent_operation(
                    ConcurrentOperationKey::Task {
                        task_id: self.task_id.clone(),
                        operation_type: operation_type.clone(),
                    },
                    None,
                );
            }
            None => {
                // No need to do anything, no operation to remove
            }
        }
    }

    fn stop_all_concurrent_operations(&self) {
        [OperationType::Work, OperationType::Check].into_iter().for_each(|operation_type| {
            self.stop_concurrent_operation(operation_type);
        })
    }

    fn get_concurrent_operation_status(&self, operation_type: OperationType) -> Option<ConcurrentOperationStatus> {
        self.system_state
            .lock()
            .unwrap()
            .get_concurrent_operation(&ConcurrentOperationKey::Task {
                task_id: self.task_id.clone(),
                operation_type,
            })
            .map(|operation| operation.status())
    }

    fn perform_concurrent_work<F>(&self, work: F, operation_type: OperationType, silent: bool)
    where
        F: FnOnce() -> WorkResult + Send + 'static,
    {
        let wrapper = WorkWrapper::wrap(
            self.system_state.clone(),
            self.query_task(|task| task.service_id.clone()),
            self.get_task_definition_id().0,
            silent,
            work,
        );
        self.system_state.lock().unwrap().set_concurrent_operation(
            ConcurrentOperationKey::Task {
                task_id: self.task_id.clone(),
                operation_type,
            },
            Some(ConcurrentOperationHandle::Work(wrapper)),
        );
    }

    fn register_external_process(&self, handle: Child, operation_type: OperationType) {
        let wrapper = ProcessWrapper::wrap(
            self.system_state.clone(),
            self.query_task(|task| task.service_id.clone()),
            self.get_task_definition_id().0,
            handle,
        );

        self.system_state.lock().unwrap().set_concurrent_operation(
            ConcurrentOperationKey::Task {
                task_id: self.task_id.clone(),
                operation_type,
            },
            Some(ConcurrentOperationHandle::Process(wrapper)),
        );
    }

    fn create_rhai_scope(&self) -> rhai::Scope {
        let mut scope = rhai::Scope::new();
        let state = self.system_state.lock().unwrap();

        populate_rhai_scope(&mut scope, &state, self.query_task(|task| task.service_id.clone()));

        scope
    }

    fn add_system_output(&self, output: String) {
        self.system_state
            .lock()
            .unwrap()
            .add_output(
                &OutputKey {
                    service_id: self.query_task(|task| task.service_id.clone()),
                    source_name: self.get_task_definition_id().0,
                    kind: OutputKind::System
                },
                output
            );
    }
}
