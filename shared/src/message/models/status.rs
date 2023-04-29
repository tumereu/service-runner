use std::collections::{HashMap, VecDeque};
use std::convert::Into;
use std::fmt::{Display, Formatter, Write};

use serde::{Deserialize, Serialize};
use toml::value::Index;

use crate::config::{
    Dependency as ConfigDependency,
    ExecutableEntry as ConfigExecutableEntry, HealthCheck as ConfigHealthCheck, HttpMethod as ConfigHttpMethod, Profile as ConfigProfile, RequiredState as ConfigRequiredState, ScriptedCompileConfig as ConfigScriptedCompileConfig, ScriptedRunConfig as ConfigScriptedRunConfig, Service as ConfigService};
use crate::message::models::{Profile, Service};
use crate::message::models::ServiceAction::Recompile;
use crate::write_escaped_str;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceStatus {
    pub action: ServiceAction,
    pub should_run: bool,
    pub compile_status: CompileStatus,
    pub run_status: RunStatus,
    pub show_output: bool,
    pub auto_recompile: bool,
}
impl ServiceStatus {
    pub fn from(_profile: &Profile, _service: &Service) -> ServiceStatus {
        ServiceStatus {
            should_run: true,
            action: Recompile,
            auto_recompile: true,
            compile_status: CompileStatus::None,
            run_status: RunStatus::Stopped,
            show_output: true,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum ServiceAction {
    None,
    Recompile,
    Restart,
    Stop
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CompileStatus {
    None,
    Compiling(usize),
    PartiallyCompiled(usize),
    FullyCompiled,
    Failed
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RunStatus {
    Stopped,
    Running,
    Healthy,
    Failed
}

