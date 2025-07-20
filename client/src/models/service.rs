use crate::config::ServiceDefinition;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Service {
    pub definition: ServiceDefinition,
    pub stage_statuses: HashMap<String, StageStatus>,
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
    Working,
    Ok,
    Error,
}
