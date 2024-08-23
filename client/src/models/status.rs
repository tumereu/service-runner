use std::collections::HashMap;
use std::time::Instant;


use crate::models::{AutomationEffect, AutomationMode, Profile, Service};

#[derive(Debug, Clone)]
pub struct ServiceStatus {
    pub action: ServiceAction,
    pub should_run: bool,
    pub debug: bool,
    pub compile_status: CompileStatus,
    pub run_status: RunStatus,
    pub show_output: bool,
    /// A yes/no -toggle controlling whether automation as a whole is enabled for this service or not. No automation
    /// trigger will fire if this is false.
    pub automation_enabled: bool,
    /// Specific automation modes for each automation entry in the service, accessible with their name.
    pub automation_modes: HashMap<String, AutomationMode>,
    pub pending_automations: Vec<PendingAutomation>,
}
impl ServiceStatus {
    pub fn from(_profile: &Profile, service: &Service) -> ServiceStatus {
        ServiceStatus {
            should_run: true,
            debug: false,
            action: ServiceAction::Recompile,
            automation_enabled: true,
            automation_modes: service.automation
                .iter()
                .map(|entry| (entry.name.clone(), entry.default_mode))
                .collect(),
            compile_status: CompileStatus::None,
            run_status: RunStatus::Stopped,
            show_output: true,
            pending_automations: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PendingAutomation {
    pub effect: AutomationEffect,
    pub not_before: Instant
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ServiceAction {
    None,
    Recompile,
    Restart,
}

#[derive(Debug, Clone)]
pub enum CompileStatus {
    None,
    Compiling(usize),
    PartiallyCompiled(usize),
    FullyCompiled,
    Failed,
}

#[derive(Debug, Clone)]
pub enum RunStatus {
    Stopped,
    Running,
    Healthy,
    Failed,
}
