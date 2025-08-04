use std::collections::HashMap;
use std::thread::JoinHandle;

use crate::config::{Block, Config};
use crate::models::{BlockStatus, GetBlock, OutputKey, OutputStore, Profile, Service};
use crate::runner::file_watcher::FileWatcherState;
use crate::runner::service_worker::AsyncOperationHandle;
use crate::ui::UIState;

pub struct SystemState {
    pub current_profile: Option<Profile>,
    pub output_store: OutputStore,
    pub ui: UIState,
    pub config: Config,
    pub should_exit: bool,
    pub active_threads: Vec<(String, JoinHandle<()>)>,
    pub file_watchers: Option<FileWatcherState>,
    async_operations: HashMap<BlockOperationKey, AsyncOperationHandle>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct BlockOperationKey {
    pub service_id: String,
    pub block_id: String,
    pub operation_type: OperationType,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum OperationType {
    /// Operation type used by prerequisite and health checks
    Check,
    /// Operation type used by the actual work performed by blocks
    Work
}

impl SystemState {
    pub fn new(config: Config) -> SystemState {
        SystemState {
            should_exit: false,
            current_profile: None,
            ui: UIState::new(),
            output_store: OutputStore::new(),
            active_threads: Vec::new(),
            file_watchers: None,
            async_operations: HashMap::new(),
            config,
        }
    }

    pub fn get_profile_name(&self) -> Option<&str> {
        self.current_profile.as_ref().map(|profile| profile.definition.id.as_str())
    }

    pub fn get_block_operation(&self, key: &BlockOperationKey) -> Option<&AsyncOperationHandle> {
        self.async_operations.get(key)
    }

    pub fn is_processing(&self, service_id: &str) -> bool {
        self.get_service(service_id)
            .iter()
            .flat_map(|service| {
                service.definition.blocks.iter().map(|block| block.id.clone())
                    .filter(|block_id| !matches!(service.get_block_status(&block_id), BlockStatus::Ok))
            })
            .flat_map(|block_id| [
                (block_id.clone(), OperationType::Check),
                (block_id.clone(), OperationType::Work),
            ])
            .any(|(block_id, operation_type)| {
                self.get_block_operation(&BlockOperationKey {
                    service_id: service_id.to_owned(),
                    block_id,
                    operation_type
                }).is_some()
            })
    }

    pub fn has_block_operations(&self, service_id: &str, block_id: &str) -> bool {
        [OperationType::Check, OperationType::Work].iter().any(|operation_type| {
            self.get_block_operation(&BlockOperationKey {
                service_id: service_id.to_owned(),
                block_id: block_id.to_string(),
                operation_type: operation_type.clone(),
            }).is_some()
        })
    }

    pub fn set_block_operation(&mut self, key: BlockOperationKey, process: Option<AsyncOperationHandle>) {
        match process {
            Some(process) => self.async_operations.insert(key, process),
            None => self.async_operations.remove(&key),
        };
    }

    pub fn get_service(&self, service_id: &str) -> Option<&Service> {
        self.current_profile
            .as_ref()
            .and_then(|profile| {
                profile
                    .services
                    .iter()
                    .find(|service| {
                        service.definition.id == service_id
                    })
            })
    }

    pub fn get_service_block(&self, service_id: &str, block_id: &str) -> Option<&Block> {
        self.get_service(service_id)
            .and_then(|service| service.get_block(block_id))
    }

    pub fn iter_services(&self) -> impl Iterator<Item = &Service> {
        self.current_profile
            .iter()
            .flat_map(|profile| profile.services.iter())
    }

    pub fn update_service<F>(&mut self, service_id: &str, update: F)
    where
        F: FnOnce(&mut Service),
    {
        let service_option = self.current_profile.as_mut()
            .and_then(|profile| {
                profile.services.iter_mut()
                    .find(|service| service.definition.id == service_id)
            });

        if let Some(service) = service_option {
            update(service);
        }
    }

    pub fn add_output(&mut self, key: &OutputKey, line: String) {
        self.output_store.add_output(key, line);
    }
}
