use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AutomationEntry {
    pub name: String,
    #[serde(default)]
    pub debounce_millis: u64,
    pub effects: Vec<AutomationEffect>,
    pub trigger: AutomationTrigger,
    pub default_mode: Some<AutomationDefaultMode>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AutomationDefaultMode {
    #[serde(rename = "automatic")]
    Automatic,
    #[serde(rename = "triggerable")]
    Triggerable,
    #[serde(rename = "disabled")]
    Disabled,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AutomationEffect {
    /// Compiles the targeted service. The service will be restarted if it is running.
    Compile,
    /// Starts the service if it is not currently running. Does nothing if the service is already started.
    Start,
    /// Restarts the service, stopping it if running and then starting it.
    Restart,
    /// Stops the service if it is currently running.
    Stop,
    /// The service's reset-action is performed
    Reset,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum AutomationTrigger {
    #[serde(rename = "recompiled-service")]
    RecompiledService { service: String },
    #[serde(rename = "modified-file")]
    ModifiedFile { paths: Vec<String> },
}

