use std::convert::Into;
use std::fmt::Write;

use serde::{Deserialize, Serialize};

use crate::config::{
    HealthCheck as ConfigHealthCheck, HttpMethod as ConfigHttpMethod,
    ScriptedRunConfig as ConfigScriptedRunConfig,
};
use crate::message::models::{Dependency, ExecutableEntry};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RunConfig {
    pub command: ExecutableEntry,
    pub dependencies: Vec<Dependency>,
    pub health_checks: Vec<HealthCheck>,
}
impl From<ConfigScriptedRunConfig> for RunConfig {
    fn from(value: ConfigScriptedRunConfig) -> Self {
        RunConfig {
            command: value.command.into(),
            dependencies: value.dependencies.into_iter().map(Into::into).collect(),
            health_checks: value.health_checks.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum HealthCheck {
    /// A health check in the form of a HTTP response, made to the given URL with the given HTTP method. The check is
    /// considered OK if something responds to the call within [timeout_millis] milliseconds with a status of [status]
    Http {
        url: String,
        method: HttpMethod,
        timeout_millis: u64,
        status: u16,
    },
    /// A health check in the form of an open port. The check is considered OK if the given [port] is is listening in
    /// the OS.
    Port { port: u16 },
}
impl From<ConfigHealthCheck> for HealthCheck {
    fn from(value: ConfigHealthCheck) -> Self {
        match value {
            ConfigHealthCheck::Http {
                url,
                method,
                timeout_millis,
                status,
            } => HealthCheck::Http {
                url,
                method: method.into(),
                timeout_millis,
                status,
            },
            ConfigHealthCheck::Port { port } => HealthCheck::Port { port },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum HttpMethod {
    GET,
    POST,
    PATCH,
    PUT,
    OPTIONS,
    DELETE,
}
impl From<ConfigHttpMethod> for HttpMethod {
    fn from(value: ConfigHttpMethod) -> Self {
        match value {
            ConfigHttpMethod::GET => HttpMethod::GET,
            ConfigHttpMethod::POST => HttpMethod::POST,
            ConfigHttpMethod::PATCH => HttpMethod::PATCH,
            ConfigHttpMethod::PUT => HttpMethod::PUT,
            ConfigHttpMethod::OPTIONS => HttpMethod::OPTIONS,
            ConfigHttpMethod::DELETE => HttpMethod::DELETE,
        }
    }
}
