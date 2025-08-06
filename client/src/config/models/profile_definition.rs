use serde_derive::{Deserialize, Serialize};
use crate::config::TaskDefinition;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ProfileDefinition {
    pub id: String,
    pub workdir: String,
    pub services: Vec<ServiceRef>,
    #[serde(default)]
    pub tasks: Vec<TaskDefinition>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ServiceRef {
    pub id: String,
}
