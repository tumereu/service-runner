use std::convert::Into;
use serde::{Deserialize, Serialize};

use crate::model::config::Service as ConfigService;
use crate::model::message::models::{AutoCompileConfig, CompileConfig, ExecutableEntry, RunConfig};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Service {
    pub name: String,
    pub dir: Option<String>,
    pub compile: Option<CompileConfig>,
    pub run: Option<RunConfig>,
    pub reset: Vec<ExecutableEntry>,
    pub autocompile: Option<AutoCompileConfig>,
}
impl From<ConfigService> for Service {
    fn from(value: ConfigService) -> Self {
        match value {
            ConfigService::Scripted {
                name,
                dir,
                compile,
                run,
                reset,
                autocompile,
            } => Service {
                name,
                dir: dir.into(),
                compile: compile.map(Into::into),
                run: run.map(Into::into),
                reset: reset.into_iter().map(Into::into).collect(),
                autocompile: autocompile.map(Into::into),
            },
        }
    }
}
