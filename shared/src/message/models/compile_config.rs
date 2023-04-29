use std::collections::{HashMap, VecDeque};
use std::convert::Into;
use std::fmt::{Display, Formatter, Write};

use serde::{Deserialize, Serialize};
use toml::value::Index;

use crate::config::{
    HttpMethod as ConfigHttpMethod,
    ExecutableEntry as ConfigExecutableEntry, Profile as ConfigProfile, Service as ConfigService, ScriptedRunConfig as ConfigScriptedRunConfig, HealthCheck as ConfigHealthCheck, Dependency as ConfigDependency, RequiredState as ConfigRequiredState, ScriptedCompileConfig as ConfigScriptedCompileConfig};
use crate::message::models::{Dependency, ExecutableEntry};
use crate::message::models::ServiceAction::{Recompile};
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


