use std::fmt::Write;

use serde::{Deserialize, Serialize};

use crate::message::models::ServiceAction::Recompile;
use crate::message::models::{Profile, Service};

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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CompileStatus {
    None,
    Compiling(usize),
    PartiallyCompiled(usize),
    FullyCompiled,
    Failed,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RunStatus {
    Stopped,
    Running,
    Healthy,
    Failed,
}
