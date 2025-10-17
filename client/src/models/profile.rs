use crate::config::{AutomationDefinitionId, ProfileDefinition, ServiceDefinition, ServiceId, TaskDefinition, TaskDefinitionId, TaskStep};
use crate::models::task::Task;
use crate::models::{Automation, AutomationStatus, Service, TaskId};
use log::error;
use std::collections::VecDeque;
use std::convert::Into;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct Profile {
    pub definition: ProfileDefinition,
    pub services: Vec<Service>,
    pub running_tasks: VecDeque<Task>,
    pub all_task_definitions: Vec<(TaskDefinition, Option<ServiceId>)>,
    pub automations: Vec<Automation>,
    pub automation_enabled: bool,
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

        let all_task_definitions: Vec<(TaskDefinition, Option<ServiceId>)> = profile.tasks.iter().map(|task_def| (task_def.clone(), None))
            .chain(
                services.iter().flat_map(|service| {
                    service.definition.tasks.iter().map(|task_def| (task_def.clone(), Some(service.definition.id.clone())))
                })
            ).collect();
        
        let profile_automations: Vec<Automation> = profile.automation.iter().map(|automation| {
            (automation.clone(), None).into()
        }).collect();

        Profile {
            definition: profile,
            services,
            running_tasks: VecDeque::new(),
            all_task_definitions,
            automations: profile_automations,
            automation_enabled: true,
        }
    }
    
    pub fn spawn_task(&mut self, task_definition_id: &TaskDefinitionId, service_id: Option<ServiceId>) {
        let task_def = match service_id.as_ref() { 
            None => self.definition.tasks.iter().find(|task| &task.id == task_definition_id),
            Some(service_id) => self.services
                .iter().find(|service| &service.definition.id == service_id)
                .and_then(|service| {
                    service.definition.tasks.iter().find(|task| &task.id == task_definition_id)
                })
        };
        
        if let Some(task_def) = task_def {
            let new_id = self.running_tasks.iter().last()
                .map(|task| task.id.0 + 1)
                .unwrap_or(1);
            
            self.running_tasks.push_back(Task {
                id: TaskId(new_id),
                definition_id: Some(task_def.id.clone()),
                name: task_def.id.0.clone(),
                steps: task_def.steps.clone(),
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

    pub fn spawn_inline_task(&mut self, service_id: Option<ServiceId>, steps: Vec<TaskStep>, name: String) {
        let new_id = self.running_tasks.iter().last()
            .map(|task| task.id.0 + 1)
            .unwrap_or(1);

        self.running_tasks.push_back(Task {
            id: TaskId(new_id),
            definition_id: None,
            name,
            steps,
            status: Default::default(),
            service_id: service_id.clone(),
            action: None,
        });
    }

    pub fn update_automation<F>(&mut self, id: &AutomationDefinitionId, update: F)
    where
            for<'a> F: FnOnce(&'a mut Automation),
    {
        if let Some(automation) = self.automations.iter_mut()
            .find(|automation| &automation.definition_id == id) {
            update(automation);
        }
    }
}