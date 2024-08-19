use std::convert::Into;
use serde::{Deserialize, Serialize};

use crate::config::ScriptedCompileConfig as ConfigScriptedCompileConfig;
use crate::models::{Dependency, ExecutableEntry};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CompileConfig {
    pub commands: Vec<ExecutableEntry>,
    pub dependencies: Vec<Dependency>,
}
impl From<ConfigScriptedCompileConfig> for CompileConfig {
    fn from(value: ConfigScriptedCompileConfig) -> Self {
        CompileConfig {
            commands: value.commands.into_iter().map(Into::into).collect(),
            dependencies: value.dependencies.into_iter().map(Into::into).collect(),
        }
    }
}
