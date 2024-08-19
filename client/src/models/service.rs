use std::convert::Into;
use serde::{Deserialize, Serialize};

use crate::config::ServiceDefinition as ConfigService;
use crate::models::{AutomationEntry, CompileConfig, ExecutableEntry, RunConfig};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Service {
    pub name: String,
    pub dir: Option<String>,
    pub compile: Option<CompileConfig>,
    pub run: Option<RunConfig>,
    pub reset: Vec<ExecutableEntry>,
    pub automation: Vec<AutomationEntry>,
}
impl From<ConfigService> for Service {
    fn from(value: ConfigService) -> Self {
        Service {
            name: value.name,
            dir: value.dir.into(),
            compile: value.compile.map(Into::into),
            run: value.run.map(Into::into),
            reset: value.reset.into_iter().map(Into::into).collect(),
            automation: value.automation.into_iter().map(Into::into).collect(),
        }
    }
}
