use crate::config::{AutomationAction, AutomationDefinition, AutomationDefinitionId, AutomationTrigger, ServiceId};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct Automation {
    pub definition_id: AutomationDefinitionId,
    pub service_id: Option<ServiceId>,
    pub status: AutomationStatus,
    pub last_triggered: Option<Instant>,
    pub debounce: Duration,
    pub action: AutomationAction,
    pub triggers: Vec<AutomationTrigger>,
    
}
impl From<(AutomationDefinition, Option<ServiceId>)> for Automation {
    fn from((definition, service_id): (AutomationDefinition, Option<ServiceId>)) -> Self {
        Self {
            definition_id: definition.id,
            service_id,
            status: if definition.enabled {
                AutomationStatus::Active
            } else {
                AutomationStatus::Disabled
            },
            last_triggered: None,
            debounce: definition.debounce,
            action: definition.action,
            triggers: definition.triggers,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AutomationStatus {
    Disabled,
    Active,
    Error,
}
