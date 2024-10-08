use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AutomationEntry {
    pub name: String,
    #[serde(default)]
    pub debounce_millis: u64,
    pub effects: Vec<AutomationEffect>,
    pub trigger: AutomationTrigger,
    #[serde(default)]
    pub default_mode: Option<AutomationDefaultMode>,
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
    #[serde(rename = "recompile")]
    Recompile,
    /// Starts the service if it is not currently running. Does nothing if the service is already started.
    #[serde(rename = "start")]
    Start,
    /// Restarts the service, stopping it if running and then starting it.
    #[serde(rename = "restart")]
    Restart,
    /// Stops the service if it is currently running.
    #[serde(rename = "stop")]
    Stop,
    /// The service's reset-action is performed
    #[serde(rename = "reset")]
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

