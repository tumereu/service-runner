use serde_derive::{Deserialize, Serialize};
use crate::config::{DependencyEntry, ExecutableEntry, HealthCheckConfig, PartialExecutableEntry};
use crate::models::AutomationEntry;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ServiceDefinition {
    pub name: String,
    pub dir: String,
    pub compile: Option<ScriptedCompileConfig>,
    pub run: Option<ScriptedRunConfig>,
    #[serde(default = "Vec::new")]
    pub reset: Vec<ExecutableEntry>,
    #[serde(default = "Vec::new")]
    pub automation: Vec<AutomationEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ScriptedCompileConfig {
    pub commands: Vec<ExecutableEntry>,
    #[serde(default)]
    pub dependencies: Vec<DependencyEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ScriptedRunConfig {
    pub command: ExecutableEntry,
    #[serde(default)]
    pub debug: PartialExecutableEntry,
    #[serde(default)]
    pub dependencies: Vec<DependencyEntry>,
    #[serde(default)]
    pub health_check: Option<HealthCheckConfig>,
}
