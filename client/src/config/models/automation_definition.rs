use crate::config::{ServiceId, TaskDefinitionId, TaskStep};
use serde_derive::{Deserialize, Serialize};
use std::time::Duration;
use derive_more::Display;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct AutomationDefinition {
    pub id: AutomationDefinitionId,
    #[serde(default, with = "humantime_serde")]
    pub debounce: Duration,
    pub action: AutomationAction,
    pub triggers: Vec<AutomationTrigger>,
    #[serde(default = "serde_aux::prelude::bool_true")]
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Display, Clone, Ord, PartialOrd, PartialEq, Eq, Hash)]
pub struct AutomationDefinitionId(pub String);

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum AutomationAction {
    #[serde(rename = "run-task")]
    RunOwnTask {
        id: TaskDefinitionId,
    },
    #[serde(rename = "run-any-task")]
    RunAnyTask {
        id: TaskDefinitionId,
        #[serde(default)]
        service: Option<ServiceId>,
    },
    #[serde(rename = "inline-task")]
    InlineTask { steps: Vec<TaskStep> },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged, deny_unknown_fields)]
pub enum AutomationTrigger {
    RhaiQuery { becomes_true: String },
    FileModified { file_modified: String },
}
