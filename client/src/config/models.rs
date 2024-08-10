use std::collections::HashMap;
use Vec;

use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Config {
    pub conf_dir: String,
    pub services: Vec<ServiceDefinition>,
    pub profiles: Vec<ProfileDefinition>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ServiceDefinition {
    pub name: String,
    pub dir: String,
    pub compile: Option<ScriptedCompileConfig>,
    pub run: Option<ScriptedRunConfig>,
    #[serde(default = "Vec::new")]
    pub reset: Vec<ExecutableEntry>,
    // TODO rework into generic "triggers" entry
    pub autocompile: Option<AutoCompileConfig>,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct HealthCheckConfig {
    pub timeout_millis: u64,
    pub checks: Vec<HealthCheck>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum HealthCheck {
    #[serde(rename = "http")]
    Http {
        url: String,
        method: HttpMethod,
        timeout_millis: u64,
        status: u16,
    },
    #[serde(rename = "port")]
    Port { port: u16 },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum HttpMethod {
    GET,
    POST,
    PATCH,
    PUT,
    DELETE,
    OPTIONS,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ExecutableEntry {
    pub executable: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct PartialExecutableEntry {
    pub executable: Option<String>,
    #[serde(default)]
    pub args: Option<Vec<String>>,
    #[serde(default)]
    pub env: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct AutoCompileConfig {
    pub mode: AutoCompileMode,
    pub triggers: Vec<AutoCompileTrigger>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AutoCompileMode {
    #[serde(rename = "automatic")]
    Automatic,
    #[serde(rename = "triggered")]
    Custom,
    #[serde(rename = "disabled")]
    Disabled,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum AutoCompileTrigger {
    #[serde(rename = "recompiled-service")]
    RecompiledService { service: String },
    #[serde(rename = "modified-file")]
    ModifiedFile { paths: Vec<String> },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ProfileDefinition {
    pub name: String,
    pub services: Vec<ServiceDefinition>,
}
impl ProfileDefinition {
    pub fn includes(&self, service: &ServiceDefinition) -> bool {
        self.services
            .iter()
            .any(|reference| reference.references(service))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ServiceRef {
    pub name: String,
}

impl ServiceRef {
    pub fn references(&self, service: &ServiceDefinition) -> bool {
        service.name() == &self.name
    }
}

fn default_server_executable() -> String {
    return String::from("./server");
}
