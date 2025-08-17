use std::time::Instant;
use crate::config::AutomationDefinition;

#[derive(Debug, Clone)]
pub struct Automation {
    pub definition: AutomationDefinition,
    pub status: AutomationStatus
}

#[derive(Debug, Clone)]
pub enum AutomationStatus {
    Disabled,
    Idle,
    PendingTrigger { time: Instant },
    Error
}