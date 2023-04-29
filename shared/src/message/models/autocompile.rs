use std::convert::Into;
use std::fmt::Write;

use serde::{Deserialize, Serialize};

use crate::config::{
    AutoCompileConfig as ConfigAutoCompileConfig, AutoCompileMode as ConfigAutoCompileMode,
    AutoCompileTrigger as ConfigAutoCompileTrigger,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AutoCompileConfig {
    pub mode: AutoCompileMode,
    pub triggers: Vec<AutoCompileTrigger>,
}
impl From<ConfigAutoCompileConfig> for AutoCompileConfig {
    fn from(value: ConfigAutoCompileConfig) -> Self {
        AutoCompileConfig {
            mode: value.mode.into(),
            triggers: value.triggers.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AutoCompileMode {
    AUTOMATIC,
    TRIGGERED,
    DISABLED,
}
impl From<ConfigAutoCompileMode> for AutoCompileMode {
    fn from(value: ConfigAutoCompileMode) -> Self {
        match value {
            ConfigAutoCompileMode::AUTOMATIC => AutoCompileMode::AUTOMATIC,
            ConfigAutoCompileMode::DISABLED => AutoCompileMode::DISABLED,
            ConfigAutoCompileMode::TRIGGERED => AutoCompileMode::TRIGGERED,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AutoCompileTrigger {
    RecompiledService { service: String },
    ModifiedFile { paths: Vec<String> },
}
impl From<ConfigAutoCompileTrigger> for AutoCompileTrigger {
    fn from(value: ConfigAutoCompileTrigger) -> Self {
        match value {
            ConfigAutoCompileTrigger::RecompiledService { service } => {
                AutoCompileTrigger::RecompiledService { service }
            }
            ConfigAutoCompileTrigger::ModifiedFile { paths } => {
                AutoCompileTrigger::ModifiedFile { paths }
            }
        }
    }
}
