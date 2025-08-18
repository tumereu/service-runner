use crate::config::{
    AutomationDefinitionId, Block, BlockId, Config, ServiceId, TaskDefinition, TaskDefinitionId,
};
use crate::models::{
    Automation, BlockStatus, GetBlock, OutputKey, OutputStore, Profile, Service, Task, TaskId,
};
use crate::runner::service_worker::ConcurrentOperationHandle;
use std::collections::HashMap;
use std::thread::JoinHandle;

pub struct SystemState {
    pub current_profile: Option<Profile>,
    pub output_store: OutputStore,
    pub config: Config,
    pub should_exit: bool,
    pub active_threads: Vec<(String, JoinHandle<()>)>,
    concurrent_operations: HashMap<ConcurrentOperationKey, ConcurrentOperationHandle>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ConcurrentOperationKey {
    Block {
        service_id: ServiceId,
        block_id: BlockId,
        operation_type: OperationType,
    },
    Task {
        task_id: TaskId,
    },
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum OperationType {
    /// Operation type used by prerequisite and health checks
    Check,
    /// Operation type used by the actual work performed by blocks
    Work,
}

impl SystemState {
    pub fn new(config: Config) -> SystemState {
        SystemState {
            should_exit: false,
            current_profile: None,
            output_store: OutputStore::new(),
            active_threads: Vec::new(),
            concurrent_operations: HashMap::new(),
            config,
        }
    }

    pub fn select_profile(&mut self, definition_id: &str) {
        self.current_profile = Some(Profile::new(
            self.config
                .profiles
                .iter()
                .find(|def| def.id == definition_id)
                .expect(&format!("No definition found with id {definition_id}"))
                .clone(),
            &self.config.services,
        ))
    }

    pub fn get_profile_name(&self) -> Option<&str> {
        self.current_profile
            .as_ref()
            .map(|profile| profile.definition.id.as_str())
    }

    pub fn get_concurrent_operation(
        &self,
        key: &ConcurrentOperationKey,
    ) -> Option<&ConcurrentOperationHandle> {
        self.concurrent_operations.get(key)
    }

    pub fn is_processing(&self, service_id: &ServiceId) -> bool {
        // TODO if a block is checking prereqs and has failed, do not count it as processing
        self.get_service(service_id)
            .iter()
            .flat_map(|service| {
                service
                    .definition
                    .blocks
                    .iter()
                    .map(|block| block.id.clone())
                    .filter(|block_id| {
                        !matches!(service.get_block_status(&block_id), BlockStatus::Ok)
                    })
            })
            .flat_map(|block_id| {
                [
                    (block_id.clone(), OperationType::Check),
                    (block_id.clone(), OperationType::Work),
                ]
            })
            .any(|(block_id, operation_type)| {
                self.get_concurrent_operation(&ConcurrentOperationKey::Block {
                    service_id: service_id.to_owned(),
                    block_id,
                    operation_type,
                })
                .is_some()
            })
    }

    pub fn has_block_operations(&self, service_id: &ServiceId, block_id: &BlockId) -> bool {
        [OperationType::Check, OperationType::Work]
            .iter()
            .any(|operation_type| {
                self.get_concurrent_operation(&ConcurrentOperationKey::Block {
                    service_id: service_id.clone(),
                    block_id: block_id.clone(),
                    operation_type: operation_type.clone(),
                })
                .is_some()
            })
    }

    pub fn set_concurrent_operation(
        &mut self,
        key: ConcurrentOperationKey,
        process: Option<ConcurrentOperationHandle>,
    ) {
        match process {
            Some(process) => self.concurrent_operations.insert(key, process),
            None => self.concurrent_operations.remove(&key),
        };
    }

    pub fn get_service(&self, service_id: &ServiceId) -> Option<&Service> {
        self.current_profile.as_ref().and_then(|profile| {
            profile
                .services
                .iter()
                .find(|service| &service.definition.id == service_id)
        })
    }

    pub fn get_task(&self, id: &TaskId) -> Option<&Task> {
        self.current_profile
            .as_ref()
            .and_then(|profile| profile.running_tasks.iter().find(|task| task.id == *id))
    }

    pub fn get_task_definition(
        &self,
        id: &TaskDefinitionId,
        service_id: Option<ServiceId>,
    ) -> Option<&TaskDefinition> {
        let result = self.current_profile.as_ref().and_then(|profile| {
            profile
                .all_task_definitions
                .iter()
                .find(|(definition, service)| definition.id == *id && *service == service_id)
                .map(|(definition, _)| definition)
        });

        result
    }

    pub fn get_service_block(&self, service_id: &ServiceId, block_id: &BlockId) -> Option<&Block> {
        self.get_service(service_id)
            .and_then(|service| service.get_block(block_id))
    }

    pub fn iter_services(&self) -> impl Iterator<Item = &Service> {
        self.current_profile
            .iter()
            .flat_map(|profile| profile.services.iter())
    }

    pub fn update_service<F>(&mut self, service_id: &ServiceId, update: F)
    where
        for<'a> F: FnOnce(&'a mut Service),
    {
        let service_option = self.current_profile.as_mut().and_then(|profile| {
            profile
                .services
                .iter_mut()
                .find(|service| &service.definition.id == service_id)
        });

        if let Some(service) = service_option {
            update(service);
        }
    }

    pub fn query_service<F, R>(&self, service_id: &ServiceId, query: F) -> Option<R>
    where
        for<'a> F: FnOnce(&'a Service) -> R,
        R: 'static,
    {
        let service_option = self.current_profile.as_ref().and_then(|profile| {
            profile
                .services
                .iter()
                .find(|service| &service.definition.id == service_id)
        });

        if let Some(service) = service_option {
            Some(query(service))
        } else {
            None
        }
    }

    pub fn query_automation<R, F>(
        &self,
        def_id: &AutomationDefinitionId,
        service_id: &Option<ServiceId>,
        query: F,
    ) -> Option<R>
    where
        for<'a> F: FnOnce(&'a Automation) -> R,
        R: 'static,
    {
        let automation = if let Some(service_id) = service_id {
            self.current_profile
                .iter()
                .flat_map(|profile| &profile.services)
                .filter(|service| service.definition.id == *service_id)
                .flat_map(|service| &service.automations)
                .find(|automation| automation.definition.id == *def_id)
        } else {
            self.current_profile
                .as_ref()
                .iter()
                .flat_map(|profile| &profile.automations)
                .find(|automation| automation.definition.id == *def_id)
        };

        if let Some(automation) = automation {
            Some(query(automation))
        } else {
            None
        }
    }

    pub fn update_automation<F>(
        &mut self,
        def_id: &AutomationDefinitionId,
        service_id: &Option<ServiceId>,
        update: F,
    ) where
        for<'a> F: FnOnce(&'a mut Automation),
    {
        let automation = if let Some(service_id) = service_id {
            self.current_profile
                .iter_mut()
                .flat_map(|profile| profile.services.iter_mut())
                .filter(|service| service.definition.id == *service_id)
                .flat_map(|service| service.automations.iter_mut())
                .find(|automation| automation.definition.id == *def_id)
        } else {
            self.current_profile
                .iter_mut()
                .flat_map(|profile| profile.automations.iter_mut())
                .find(|automation| automation.definition.id == *def_id)
        };

        if let Some(automation) = automation {
            update(automation);
        }
    }

    pub fn update_all_services<F>(&mut self, update: F)
    where
        F: Fn((usize, &mut Service)),
    {
        if let Some(profile) = self.current_profile.as_mut() {
            profile.services.iter_mut().enumerate().for_each(update);
        }
    }

    pub fn update_task<F>(&mut self, id: &TaskId, update: F)
    where
        F: FnOnce(&mut Task),
    {
        let task_option = self
            .current_profile
            .as_mut()
            .and_then(|profile| profile.running_tasks.iter_mut().find(|task| task.id == *id));

        if let Some(task) = task_option {
            update(task);
        }
    }

    pub fn add_output(&mut self, key: &OutputKey, line: String) {
        self.output_store.add_output(key, line);
    }
}
