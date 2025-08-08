use crate::config::{Block, Config, TaskDefinition, TaskDefinitionId};
use crate::models::{
    BlockStatus, GetBlock, OutputKey, OutputStore, Profile, Service, Task, TaskId,
};
use crate::runner::file_watcher::FileWatcherState;
use crate::runner::service_worker::ConcurrentOperationHandle;
use crate::ui::UIState;
use std::collections::{BTreeMap, VecDeque};
use std::sync::RwLock;
use std::thread::JoinHandle;
use crate::runner::service_worker::concurrent_operation::ConcurrentOperationStatus;

pub struct SystemState(RwLock<InnerState>);

struct InnerState {
    current_profile: Option<Profile>,
    output_store: OutputStore,
    ui: UIState,
    config: Config,
    should_exit: bool,
    active_threads: Vec<(String, JoinHandle<()>)>,
    file_watchers: Option<FileWatcherState>,
    concurrent_operations: BTreeMap<ConcurrentOperationKey, ConcurrentOperationHandle>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum ConcurrentOperationKey {
    Block {
        service_id: String,
        block_id: String,
        operation_type: OperationType,
    },
    Task {
        task_id: TaskId,
    },
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum OperationType {
    /// Operation type used by prerequisite and health checks
    Check,
    /// Operation type used by the actual work performed by blocks
    Work,
}

impl SystemState {
    pub fn new(config: Config) -> SystemState {
        SystemState(RwLock::new(InnerState {
            should_exit: false,
            current_profile: None,
            ui: UIState::new(),
            output_store: OutputStore::new(),
            active_threads: Vec::new(),
            file_watchers: None,
            concurrent_operations: BTreeMap::new(),
            config,
        }))
    }

    pub fn query_profile<R, F>(&self, query: F) -> Option<R>
    where
        for<'a> F: FnOnce(&'a Profile) -> R,
        R: 'static,
    {
        let state = self.0.read().unwrap();
        state.current_profile.as_ref().map(query)
    }

    pub fn update_profile<R, F>(&self, update: F) -> Option<R>
    where
            for<'a> F: FnOnce(&'a mut Profile) -> R,
            R: 'static,
    {
        let mut state = self.0.write().unwrap();
        state.current_profile.as_mut().map(update)
    }

    pub fn query_output<R, F>(&self, query: F) -> R
    where
        for<'a> F: FnOnce(&'a OutputStore) -> R,
        R: 'static,
    {
        let state = self.0.read().unwrap();
        query(&state.output_store)
    }

    pub fn update_output<R, F>(&self, update: F) -> R
    where
        for<'a> F: FnOnce(&'a mut OutputStore) -> R,
        R: 'static,
    {
        let mut state = self.0.write().unwrap();
        update(&mut state.output_store)
    }

    pub fn query_ui<R, F>(&self, query: F) -> R
    where
        for<'a> F: FnOnce(&'a UIState) -> R,
        R: 'static,
    {
        let state = self.0.read().unwrap();
        query(&state.ui)
    }

    pub fn update_ui<R, F>(&self, update: F) -> R
    where
        for<'a> F: FnOnce(&'a mut UIState) -> R,
        R: 'static,
    {
        let mut state = self.0.write().unwrap();
        update(&mut state.ui)
    }

    pub fn query_config<R, F>(&self, query: F) -> R
    where
        for<'a> F: FnOnce(&'a Config) -> R,
        R: 'static,
    {
        let state = self.0.read().unwrap();
        query(&state.config)
    }

    pub fn set_config(&self, new_config: Config) {
        let mut state = self.0.write().unwrap();
        state.config = new_config;
    }

    pub fn should_exit(&self) -> bool {
        let state = self.0.read().unwrap();
        state.should_exit
    }

    pub fn exit(&self) {
        let mut state = self.0.write().unwrap();
        state.should_exit = true;
    }

    pub fn query_system<R, F>(&self, query: F) -> R
    where
        for<'a> F: FnOnce(&'a InnerState) -> R,
        R: 'static,
    {
        let state = self.0.read().unwrap();
        query(&state)
    }

    pub fn query_concurrent_operation<R, F>(&self, key: &ConcurrentOperationKey, query: F) -> R
    where
            for<'a> F: FnOnce(&'a Option<&ConcurrentOperationHandle>) -> R,
            R: 'static,
    {
        let state = self.0.read().unwrap();
        query(&state.concurrent_operations.get(key))
    }

    pub fn get_concurrent_operation_status(&self, key: &ConcurrentOperationKey) -> Option<ConcurrentOperationStatus>
    {
        self.query_concurrent_operation(key, |handle| handle.map(|operation| operation.status()))
    }

    pub fn is_processing(&self, service_id: &str) -> bool {
        self.query_service(service_id, |service| {
            service
                .definition
                .blocks
                .iter()
                .map(|block| block.id.clone())
                .filter(|block_id| {
                    !matches!(service.get_block_status(&block_id), BlockStatus::Ok)
                })
                .flat_map(|block_id| {
                    [
                        (block_id.clone(), OperationType::Check),
                        (block_id.clone(), OperationType::Work),
                    ]
                })
                .any(|(block_id, operation_type)| {
                    self.get_concurrent_operation_status(
                        &ConcurrentOperationKey::Block {
                            service_id: service_id.to_owned(),
                            block_id,
                            operation_type,
                        }
                    ).is_some()
                })
        }).unwrap_or(false)
    }

    pub fn has_block_operations(&self, service_id: &str, block_id: &str) -> bool {
        [OperationType::Check, OperationType::Work]
            .into_iter()
            .any(|operation_type| {
                self.get_concurrent_operation_status(
                    &ConcurrentOperationKey::Block {
                        service_id: service_id.to_owned(),
                        block_id: block_id.to_owned(),
                        operation_type,
                    }
                ).is_some()
            })
    }

    pub fn query_service<R, F>(&self, service_id: &str, query: F) -> Option<R>
    where
            for<'a> F: FnOnce(&'a Service) -> R,
            R: 'static,
    {
        let state = self.0.read().unwrap();
        state.current_profile.as_ref().and_then(|profile| {
            profile.services.iter().find(|service| service.definition.id == service_id)
        }).map(query)
    }

    pub fn update_service<R, F>(&self, service_id: &str, update: F) -> Option<R>
    where
            for<'a> F: FnOnce(&'a mut Service) -> R,
            R: 'static,
    {
        let mut state = self.0.write().unwrap();
        state.current_profile.as_mut().and_then(|profile| {
            profile.services.iter_mut().find(|service| service.definition.id == service_id)
        }).map(update)
    }

    pub fn query_services<R, F>(&self, query: F) -> Option<R>
    where
            for<'a> F: FnOnce(&'a Vec<Service>) -> R,
            R: 'static,
    {
        let state = self.0.read().unwrap();
        state.current_profile.as_ref().map(|profile| query(&profile.services))
    }

    pub fn set_concurrent_operation(
        &mut self,
        key: ConcurrentOperationKey,
        process: Option<ConcurrentOperationHandle>,
    ) {
        let mut state = self.0.write().unwrap();
        match process {
            Some(process) => state.concurrent_operations.insert(key, process),
            None => state.concurrent_operations.remove(&key),
        };
    }

    pub fn query_task<R, F>(&self, task_id: &TaskId, query: F) -> Option<R>
    where
            for<'a> F: FnOnce(&'a Task) -> R,
            R: 'static,
    {
        let state = self.0.read().unwrap();
        state.current_profile.as_ref().and_then(|profile| {
            profile.tasks.iter().find(|task| task.id == *task_id)
        }).map(query)
    }

    pub fn update_task<R, F>(&self, task_id: &TaskId, update: F) -> Option<R>
    where
            for<'a> F: FnOnce(&'a mut Task) -> R,
            R: 'static,
    {
        let mut state = self.0.write().unwrap();
        state.current_profile.as_mut().and_then(|profile| {
            profile.tasks.iter_mut().find(|task| task.id == *task_id)
        }).map(update)
    }

    pub fn query_tasks<R, F>(&self, query: F) -> Option<R>
    where
            for<'a> F: FnOnce(&'a VecDeque<Task>) -> R,
            R: 'static,
    {
        let state = self.0.read().unwrap();
        state.current_profile.as_ref().map(|profile| query(&profile.tasks))
    }

    pub fn query_task_definition<R, F>(
        &self,
        id: &TaskDefinitionId,
        service_id: Option<String>,
        query: F) -> Option<R>
    where
            for<'a> F: FnOnce(&'a TaskDefinition) -> R,
            R: 'static,
    {
        let state = self.0.read().unwrap();
        state.current_profile.as_ref().and_then(|profile| {
            profile.all_task_definitions
                .iter()
                .find(|(definition, service)| definition.id == *id && *service == service_id)
                .map(|(definition, _)| definition)
        }).map(query)
    }

    pub fn query_block<R, F>(&self, service_id: &str, block_id: &str, query: F) -> Option<R>
    where
            for<'a> F: FnOnce(&'a Block) -> R,
            R: 'static,
    {
        self.query_service(service_id, |service| service.get_block(block_id).map(query)).flatten()
    }

    pub fn add_output(&mut self, key: &OutputKey, line: String) {
        self.update_output(|output_store| {
            output_store.add_output(key, line);
        });
    }
}
