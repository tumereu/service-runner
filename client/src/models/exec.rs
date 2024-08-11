use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use crate::config::{ExecutableEntry as ConfigExecutableEntry, PartialExecutableEntry as ConfigPartialExecutableEntry};
use crate::utils::write_escaped_str;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExecutableEntry {
    pub executable: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}

impl ExecutableEntry {
    pub fn extend(&self, other: &PartialExecutableEntry) -> ExecutableEntry {
        ExecutableEntry {
            executable: other.executable.clone().unwrap_or_else(|| self.executable.clone()),
            args: other.args.clone().unwrap_or_else(|| self.args.clone()),
            // TODO combine maps
            env: other.env.clone().unwrap_or_else(|| self.env.clone()),
        }
    }
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
            f.write_str(" (env:")?;
            // Place env keys into a tmp variable so that we can sort them
            let mut env_keys: Vec<String> = self.env.keys().map(Clone::clone).collect();
            env_keys.sort();
            for key in env_keys {
                f.write_str(" ")?;
                write_escaped_str!(f, &key);
                f.write_str("=")?;
                write_escaped_str!(f, self.env.get(&key).unwrap());
            }
            f.write_str(")")?;
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PartialExecutableEntry {
    pub executable: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
}
impl From<ConfigPartialExecutableEntry> for PartialExecutableEntry {
    fn from(value: ConfigPartialExecutableEntry) -> Self {
        PartialExecutableEntry {
            executable: value.executable,
            args: value.args,
            env: value.env,
        }
    }
}