use std::collections::HashMap;
use Vec;

use serde::Deserialize;
use serde_aux::field_attributes::bool_true;

#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub conf_dir: String,
    pub services: Vec<Service>,
    pub profiles: Vec<Profile>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    pub port: u16,
    #[serde(default = "bool_true")]
    pub daemon: bool,
    #[serde(default = "default_server_executable")]
    pub executable: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum Service {
    #[serde(rename = "scripted")]
    Scripted {
        name: String,
        dir: String,
        compile: Option<ScriptedCompileConfig>,
        run: Option<ScriptedRunConfig>,
        reset: Vec<ExecutableEntry>,
    }
}

impl Service {
    pub fn name(&self) -> &String {
        match self {
            Service::Scripted { name, .. } => &name,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ScriptedCompileConfig {
    pub commands: Vec<ExecutableEntry>,
    #[serde(default)]
    pub dependencies: Vec<Dependency>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ScriptedRunConfig {
    pub command: ExecutableEntry,
    #[serde(default)]
    pub dependencies: Vec<Dependency>,
    #[serde(default)]
    pub health_check: Vec<HealthCheck>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Dependency {
    pub service: String,
    pub require:  RequiredState
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub enum RequiredState {
    #[serde(rename = "compiled")]
    Compiled,
    #[serde(rename = "running")]
    Running
}

#[derive(Deserialize, Debug, Clone)]
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
    Port {
        port: u16
    },
}

#[derive(Deserialize, Debug, Clone)]
pub enum HttpMethod {
    GET,
    POST,
    PATCH,
    PUT,
    DELETE,
    OPTIONS,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ExecutableEntry {
    pub executable: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub name: String,
    pub services: Vec<ServiceRef>,
}
impl Profile {
    pub fn includes(&self, service: &Service) -> bool {
        self.services.iter().any(|reference| reference.references(service))
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ServiceRef {
    pub name: String,
}

impl ServiceRef {
    pub fn references(&self, service: &Service) -> bool {
        service.name() == &self.name
    }
}

fn default_server_executable() -> String {
    return String::from("./server");
}