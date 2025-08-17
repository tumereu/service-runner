use std::time::Duration;
use serde_derive::{Deserialize, Serialize};
use crate::config::{ServiceId, TaskDefinitionId, TaskStep};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct AutomationDefinition {
    pub name: String,
    #[serde(default, with = "humantime_serde")]
    pub debounce: Duration,
    pub action: AutomationAction,
    pub triggers: Vec<AutomationTrigger>,
    #[serde(default = "serde_aux::prelude::bool_true")]
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct TaskReference {
    pub task: TaskDefinitionId,
    #[serde(default)]
    pub service: Option<ServiceId>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged, deny_unknown_fields)]
pub enum AutomationAction {
    Task(TaskReference),
    Tasks(Vec<TaskReference>),
    InlineTask(Vec<TaskStep>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged, deny_unknown_fields)]
pub enum AutomationTrigger {
    RhaiQuery { becomes_true: String },
    FileModified { file_modified: String },
}

