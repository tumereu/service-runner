use serde::{Deserialize, Serialize};

use crate::model::message::models::ServiceAction::Recompile;
use crate::model::message::models::{AutoCompileMode, Profile, Service};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceStatus {
    pub action: ServiceAction,
    pub should_run: bool,
    pub debug: bool,
    pub compile_status: CompileStatus,
    pub run_status: RunStatus,
    pub show_output: bool,
    pub auto_compile: Option<AutoCompileMode>,
    pub has_uncompiled_changes: bool,
}
impl ServiceStatus {
    pub fn from(_profile: &Profile, service: &Service) -> ServiceStatus {
        ServiceStatus {
            should_run: true,
            debug: false,
            action: Recompile,
            auto_compile: service.autocompile.as_ref().map(|auto_compile| auto_compile.default_mode.clone()),
            compile_status: CompileStatus::None,
            run_status: RunStatus::Stopped,
            show_output: true,
            has_uncompiled_changes: false,
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
