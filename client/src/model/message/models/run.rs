use std::convert::Into;
use serde_derive::{Deserialize, Serialize};

use crate::model::config::{
    HealthCheck as ConfigHealthCheck, HttpMethod as ConfigHttpMethod,
    ScriptedRunConfig as ConfigScriptedRunConfig,
    HealthCheckConfig as ConfigHealthCheckConfig
};
use crate::model::message::models::{Dependency, ExecutableEntry, PartialExecutableEntry};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RunConfig {
    pub command: ExecutableEntry,
    pub debug: PartialExecutableEntry,
    pub dependencies: Vec<Dependency>,
    pub health_check: Option<HealthCheckConfig>,
}
impl From<ConfigScriptedRunConfig> for RunConfig {
    fn from(value: ConfigScriptedRunConfig) -> Self {
        RunConfig {
            command: value.command.into(),
            debug: value.debug.into(),
            dependencies: value.dependencies.into_iter().map(Into::into).collect(),
            health_check: value.health_check.map(Into::into),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HealthCheckConfig {
    pub timeout_millis: u64,
    pub checks: Vec<HealthCheck>
}
impl From<ConfigHealthCheckConfig> for HealthCheckConfig {
    fn from(value: ConfigHealthCheckConfig) -> Self {
        HealthCheckConfig {
            timeout_millis: value.timeout_millis,
            checks: value.checks.into_iter().map(Into::into).collect(),
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
