use std::collections::{BTreeMap, VecDeque};
use std::convert::Into;
use std::time::Instant;
use log::error;
use crate::config::{ProfileDefinition, ServiceDefinition, TaskDefinitionId};
use crate::models::{Service, TaskId};
use crate::models::task::{Task, TaskStatus};

#[derive(Debug, Clone)]
pub struct Profile {
    pub definition: ProfileDefinition,
    pub services: Vec<Service>,
    pub tasks: VecDeque<Task>,
    pub task_id_to_idx: BTreeMap<TaskId, usize>,
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

        Profile {
            definition: profile,
            services,
            tasks: VecDeque::new(),
            task_id_to_idx: BTreeMap::new(),
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
                start_time: Instant::now(),
                service_id: service_id.clone(),
                action: None,
            });
            if self.tasks.len() > MAX_RETAINED_TASKS {
                self.tasks.pop_front();   
            }
        } else if let Some(service_id) = service_id {
            error!("No task {task_definition_id} found in service {service_id}");
        } else {
            error!("No standalone task {task_definition_id} found in profile");
        }
    }
}

const MAX_RETAINED_TASKS: usize = 1024;