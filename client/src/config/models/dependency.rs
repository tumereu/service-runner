use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct DependencyEntry {
    pub service: String,
    pub require: RequiredState,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub enum RequiredState {
    #[serde(rename = "compiled")]
    Compiled,
    #[serde(rename = "running")]
    Running,
}
