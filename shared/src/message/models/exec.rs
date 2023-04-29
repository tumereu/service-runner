use std::collections::{HashMap, VecDeque};
use std::convert::Into;
use std::fmt::{Display, Formatter, Write};

use serde::{Deserialize, Serialize};
use toml::value::Index;

use crate::config::{
    Dependency as ConfigDependency,
    ExecutableEntry as ConfigExecutableEntry, HealthCheck as ConfigHealthCheck, HttpMethod as ConfigHttpMethod, Profile as ConfigProfile, RequiredState as ConfigRequiredState, ScriptedCompileConfig as ConfigScriptedCompileConfig, ScriptedRunConfig as ConfigScriptedRunConfig, Service as ConfigService};
use crate::message::models::ServiceAction::Recompile;
use crate::write_escaped_str;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExecutableEntry {
    pub executable: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}
impl From<ConfigExecutableEntry> for ExecutableEntry {
    fn from(value: ConfigExecutableEntry) -> Self {
        ExecutableEntry {
            executable: value.executable,
            args: value.args,
            env: value.env,
        }
    }
}
impl Display for ExecutableEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.executable)?;
        for arg in &self.args {
            f.write_str(" ")?;
            write_escaped_str!(f, arg);
        }

        if !self.env.is_empty() {
            f.write_str(" (env: ")?;
            // Place env keys into a tmp variable so that we can sort them
            let mut env_keys: Vec<String> = self.env.keys().map(Clone::clone).collect();
            env_keys.sort();
            for key in env_keys {
                write_escaped_str!(f, &key);
                f.write_str("=")?;
                write_escaped_str!(f, self.env.get(&key).unwrap());
            }
            f.write_str(")")?;
        }

        Ok(())
    }
}

