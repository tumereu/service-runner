use crate::config::{ServiceDefinition, Stage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Service {
    pub definition: ServiceDefinition,
    stage_statuses: HashMap<String, StageStatus>,
}
impl Service {
    pub fn update_stage_status(&mut self, stage: &str, status: StageStatus)
    {
        self.stage_statuses.insert(stage.to_owned(), status);
    }

    pub fn get_stage_status(&self, stage: &str) -> StageStatus
    {
        self.stage_statuses.get(stage).unwrap_or(&StageStatus::Initial).clone()
    }
}
impl From<ServiceDefinition> for Service {
    fn from(value: ServiceDefinition) -> Self {
        Service {
            definition: value,
            stage_statuses: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum StageStatus {
    Initial,
    Working {
        steps_completed: usize,
        current_step: Option<usize>,
    },
    Ok,
    Error,
}

pub trait GetStage {
    fn get_stage(&self, name: &str) -> Option<&Stage>;
}

impl GetStage for ServiceDefinition {
    fn get_stage(&self, name: &str) -> Option<&Stage> {
        self.stages.iter().find(|s| s.name == name)
    }
}
impl GetStage for Service {
    fn get_stage(&self, name: &str) -> Option<&Stage> {
        self.definition.get_stage(name)
    }
}