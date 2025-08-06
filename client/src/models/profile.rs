use std::collections::{BTreeMap, VecDeque};
use std::convert::Into;
use std::io::Read;
use std::time::Instant;
use log::error;
use crate::config::{ProfileDefinition, ServiceDefinition, TaskDefinition, TaskDefinitionId};
use crate::models::{Service, TaskId};
use crate::models::task::{Task, TaskStatus};

#[derive(Debug, Clone)]
pub struct Profile {
    pub definition: ProfileDefinition,
    pub services: Vec<Service>,
    pub tasks: VecDeque<Task>,
    pub all_task_definitions: Vec<(TaskDefinition, Option<String>)>
}
impl Profile {
    pub fn new(profile: ProfileDefinition, all_services: &Vec<ServiceDefinition>) -> Profile {
        let services: Vec<Service> = profile.services
            .iter()
            .flat_map(|service_ref| {
                all_services.iter()
                    .find(|service| &service.id == &service_ref.id)
                    .map(|service| service.to_owned().into())
                    .into_iter()
            })
            .collect();

        let all_task_definitions: Vec<(TaskDefinition, Option<String>)> = profile.tasks.iter().map(|task_def| (task_def.clone(), None))
            .chain(
                services.iter().flat_map(|service| {
                    service.definition.tasks.iter().map(|task_def| (task_def.clone(), Some(service.definition.id.clone())))
                })
            ).collect();

        Profile {
            definition: profile,
            services,
            tasks: VecDeque::new(),
            all_task_definitions,
        }
    }
    
    pub fn spawn_task(&mut self, task_definition_id: &TaskDefinitionId, service_id: Option<String>) {
        let task_def = match service_id.as_ref() { 
            None => self.definition.tasks.iter().find(|task| &task.id == task_definition_id),
            Some(service_id) => self.services
                .iter().find(|service| &service.definition.id == service_id)
                .and_then(|service| {
                    service.definition.tasks.iter().find(|task| &task.id == task_definition_id)
                })
        };
        
        if let Some(task_def) = task_def {
            let new_id = self.tasks.iter().last()
                .map(|task| task.id.0 + 1)
                .unwrap_or(1);
            
            self.tasks.push_back(Task {
                id: TaskId(new_id),
                definition_id: task_def.id.clone(),
                status: Default::default(),
                service_id: service_id.clone(),
                action: None,
            });
        } else if let Some(service_id) = service_id {
            error!("No task {task_definition_id} found in service {service_id}");
        } else {
            error!("No standalone task {task_definition_id} found in profile");
        }
    }
}