use std::time::Duration;

use serde_derive::{Deserialize, Serialize};

use crate::config::{AutomationDefinition, ExecutableEntry, Requirement};
use derive_more::Display;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceDefinition {
    pub id: ServiceId,
    pub workdir: String,
    pub blocks: Vec<Block>,
    #[serde(default = "Vec::new")]
    pub automation: Vec<AutomationDefinition>,
    #[serde(default = "Vec::new")]
    pub tasks: Vec<TaskDefinition>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Display, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ServiceId(String);
impl ServiceId {
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }
    
    pub fn inner(&self) -> &str {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub id: BlockId,
    pub status_line: StatusLine,
    #[serde(default)]
    pub health: HealthCheckConfig,
    #[serde(default)]
    pub prerequisites: Vec<Requirement>,
    #[serde(flatten)]
    pub work: WorkDefinition,
    pub resource_group: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Display, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockId(String);
impl BlockId {
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }
    
    pub fn inner(&self) -> &str {
        &self.0
    }
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