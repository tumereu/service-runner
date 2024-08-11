use std::convert::Into;
use serde::{Deserialize, Serialize};

use crate::config::{
    AutomationEntry as ConfigAutomationEntry,
    AutomationEffect as ConfigAutomationEffect,
    AutomationDefaultMode as ConfigAutomationDefaultMode,
    AutomationTrigger as ConfigAutomationTrigger,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AutomationEntry {
    pub name: String,
    pub debounce_millis: u64,
    pub effects: Vec<AutomationEffect>,
    pub trigger: AutomationTrigger,
    pub default_mode: AutomationMode,
}
impl From<ConfigAutomationEntry> for AutomationEntry {
    fn from(value: ConfigAutomationEntry) -> Self {
        AutomationEntry {
            name: value.name,
            debounce_millis: value.debounce_millis,
            effects: value.effects.into_iter().map(Into::into).collect(),
            trigger: value.trigger.into(),
            default_mode: value.default_mode.unwrap_or(ConfigAutomationDefaultMode::Automatic).into(),
        }
    }
}

/// See [ConfigAutomationEffect]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum AutomationEffect {
    Recompile,
    Start,
    Restart,
    Stop,
    Reset,
}
impl From<ConfigAutomationEffect> for AutomationEffect {
    fn from(value: ConfigAutomationEffect) -> Self {
        match value {
            ConfigAutomationEffect::Recompile => AutomationEffect::Recompile,
            ConfigAutomationEffect::Start => AutomationEffect::Start,
            ConfigAutomationEffect::Restart => AutomationEffect::Restart,
            ConfigAutomationEffect::Stop => AutomationEffect::Stop,
            ConfigAutomationEffect::Reset => AutomationEffect::Reset,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, PartialEq, Eq, Hash)]
pub enum AutomationMode {
    Automatic,
    Triggerable,
    Disabled,
}
impl From<ConfigAutomationDefaultMode> for AutomationMode {
    fn from(value: ConfigAutomationDefaultMode) -> Self {
        match value {
            ConfigAutomationDefaultMode::Automatic => AutomationMode::Automatic,
            ConfigAutomationDefaultMode::Disabled => AutomationMode::Disabled,
            ConfigAutomationDefaultMode::Triggerable => AutomationMode::Triggerable,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AutomationTrigger {
    RecompiledService { service: String },
    ModifiedFile { paths: Vec<String> },
}
impl From<ConfigAutomationTrigger> for AutomationTrigger {
    fn from(value: ConfigAutomationTrigger) -> Self {
        match value {
            ConfigAutomationTrigger::RecompiledService { service } => {
                AutomationTrigger::RecompiledService { service }
            }
            ConfigAutomationTrigger::ModifiedFile { paths } => {
                AutomationTrigger::ModifiedFile { paths }
            }
        }
    }
}
