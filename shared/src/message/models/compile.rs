use std::collections::{HashMap, VecDeque};
use std::convert::Into;
use std::fmt::{Display, Formatter, Write};

use serde::{Deserialize, Serialize};
use toml::value::Index;

use crate::config::{
    Dependency as ConfigDependency,
    ExecutableEntry as ConfigExecutableEntry, HealthCheck as ConfigHealthCheck, HttpMethod as ConfigHttpMethod, Profile as ConfigProfile, RequiredState as ConfigRequiredState, ScriptedCompileConfig as ConfigScriptedCompileConfig, ScriptedRunConfig as ConfigScriptedRunConfig, Service as ConfigService};
use crate::message::models::{Dependency, ExecutableEntry};
use crate::message::models::ServiceAction::Recompile;
use crate::write_escaped_str;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CompileConfig {
    pub commands: Vec<ExecutableEntry>,
    pub dependencies: Vec<Dependency>,
}
impl From<ConfigScriptedCompileConfig> for CompileConfig {
    fn from(value: ConfigScriptedCompileConfig) -> Self {
        CompileConfig {
            commands: value.commands.into_iter().map(Into::into).collect(),
            dependencies: value.dependencies.into_iter().map(Into::into).collect(),
        }
    }
}


