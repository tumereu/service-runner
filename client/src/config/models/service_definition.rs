use std::time::Duration;

use serde_derive::{Deserialize, Serialize};

use derive_more::{Display};
use crate::config::{AutomationEntry, ExecutableEntry, HttpMethod, Requirement};

// TODO validate :
//      - block or task ids/names should be unique
//      - ids should be at most 23 characters long to support SmartString
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceDefinition {
    pub id: String,
    pub dir: String,
    pub blocks: Vec<Block>,
    #[serde(default = "Vec::new")]
    pub automation: Vec<AutomationEntry>,
    #[serde(default = "Vec::new")]
    pub tasks: Vec<TaskDefinition>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub id: String,
    pub status_line: StatusLine,
    #[serde(default)]
    pub health: HealthCheckConfig,
    #[serde(default)]
    pub prerequisites: Vec<Requirement>,
    #[serde(flatten)]
    pub work: WorkDefinition,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct HealthCheckConfig {
    #[serde(default, with = "humantime_serde")]
    pub timeout: Duration,
    pub requirements: Vec<Requirement>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StatusLine {
    pub symbol: String,
    pub slot: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum WorkDefinition {
    #[serde(rename = "cmd-seq")]
    CommandSeq { commands: Vec<ExecutableEntry> },
    #[serde(rename = "process")]
    Process { command: ExecutableEntry },
}

#[derive(Serialize, Deserialize, Debug, Display, Clone, PartialOrd, PartialEq, Eq, Hash)]
pub struct TaskDefinitionId(pub String);

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct TaskDefinition {
    pub id: TaskDefinitionId,
    pub steps: Vec<TaskStep>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged, deny_unknown_fields)]
pub enum TaskStep {
    Command {
        #[serde(flatten)]
        command: ExecutableEntry
    },
    Action { action: String },
    Wait { 
        #[serde(with = "humantime_serde")]
        timeout: Duration,
        requirement: Requirement,
    },
}