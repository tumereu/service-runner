use crate::config::AutomationDefinition;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct Automation {
    pub definition: AutomationDefinition,
    pub status: AutomationStatus,
    pub last_triggered: Option<Instant>,
}
impl From<AutomationDefinition> for Automation {
    fn from(value: AutomationDefinition) -> Self {
        Self {
            status: if value.enabled {
                AutomationStatus::Active
            } else {
                AutomationStatus::Disabled
            },
            definition: value,
            last_triggered: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AutomationStatus {
    Disabled,
    Active,
    Error,
}
