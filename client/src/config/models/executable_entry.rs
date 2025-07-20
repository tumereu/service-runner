use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use crate::utils::write_escaped_str;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ExecutableEntry {
    pub executable: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
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