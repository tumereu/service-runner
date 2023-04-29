use std::collections::{HashMap, VecDeque};
use std::convert::Into;
use std::fmt::{Display, Formatter, Write};

use serde::{Deserialize, Serialize};
use toml::value::Index;

use crate::config::{
    HttpMethod as ConfigHttpMethod,
    ExecutableEntry as ConfigExecutableEntry, Profile as ConfigProfile, Service as ConfigService, ScriptedRunConfig as ConfigScriptedRunConfig, HealthCheck as ConfigHealthCheck, Dependency as ConfigDependency, RequiredState as ConfigRequiredState, ScriptedCompileConfig as ConfigScriptedCompileConfig};
use crate::message::models::{AutoCompileConfig, CompileConfig, ExecutableEntry, RunConfig};
use crate::write_escaped_str;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Service {
    pub name: String,
    pub dir: Option<String>,
    pub compile: Option<CompileConfig>,
    pub run: Option<RunConfig>,
    pub reset: Vec<ExecutableEntry>,
    pub autocompile: Option<AutoCompileConfig>,
}
impl From<ConfigService> for Service {
    fn from(value: ConfigService) -> Self {
        match value {
            ConfigService::Scripted { name, dir, compile, run, reset, autocompile } => {
                Service {
                    name,
                    dir: dir.into(),
                    compile: compile.map(Into::into),
                    run: run.map(Into::into),
                    reset: reset.into_iter().map(Into::into).collect(),
                    autocompile: autocompile.map(Into::into),
                }
            }
        }
    }
}
