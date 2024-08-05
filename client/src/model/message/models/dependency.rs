use std::convert::Into;
use serde::{Deserialize, Serialize};

use crate::config::{Dependency as ConfigDependency, RequiredState as ConfigRequiredState};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Dependency {
    pub service: String,
    pub requirement: RequiredState,
}
impl From<ConfigDependency> for Dependency {
    fn from(value: ConfigDependency) -> Self {
        Dependency {
            service: value.service,
            requirement: value.require.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RequiredState {
    Compiled,
    Running,
}
impl From<ConfigRequiredState> for RequiredState {
    fn from(value: ConfigRequiredState) -> Self {
        match value {
            ConfigRequiredState::Compiled => RequiredState::Compiled,
            ConfigRequiredState::Running => RequiredState::Running,
        }
    }
}
