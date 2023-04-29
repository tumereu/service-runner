use std::collections::{HashMap, VecDeque};
use std::convert::Into;
use std::fmt::{Display, Formatter, Write};

use serde::{Deserialize, Serialize};
use toml::value::Index;

use crate::config::{
    HttpMethod as ConfigHttpMethod,
    ExecutableEntry as ConfigExecutableEntry, Profile as ConfigProfile, Service as ConfigService, ScriptedRunConfig as ConfigScriptedRunConfig, HealthCheck as ConfigHealthCheck, Dependency as ConfigDependency, RequiredState as ConfigRequiredState, ScriptedCompileConfig as ConfigScriptedCompileConfig};
use crate::message::models::ServiceAction::{Recompile};
use crate::write_escaped_str;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Dependency {
    pub service: String,
    pub requirement: RequiredState
}
impl From<ConfigDependency> for Dependency {
    fn from(value: ConfigDependency) -> Self {
        Dependency {
            service: value.service,
            requirement: value.require.into()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RequiredState {
    Compiled,
    Running
}
impl From<ConfigRequiredState> for RequiredState {
    fn from(value: ConfigRequiredState) -> Self {
        match value {
            ConfigRequiredState::Compiled => RequiredState::Compiled,
            ConfigRequiredState::Running => RequiredState::Running
        }
    }
}

