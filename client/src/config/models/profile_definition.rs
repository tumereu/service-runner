use crate::config::{AutomationDefinition, ServiceId, TaskDefinition};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ProfileDefinition {
    pub id: String,
    pub workdir: String,
    pub services: Vec<ServiceRef>,
    #[serde(default)]
    pub tasks: Vec<TaskDefinition>,
    #[serde(default = "Vec::new")]
    pub automation: Vec<AutomationDefinition>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ServiceRef {
    pub id: ServiceId,
}
