use crate::models::{Profile, ServiceAction};

pub enum Action {
    Tick,
    Shutdown,
    ActivateProfile(Profile),
    Reset(String),
    ResetAll,
    Restart(String),
    RestartAll,
    Recompile(String),
    RecompileAll,
    CycleAutomation(String),
    UpdateRun(String, bool),
    ToggleRun(String),
    ToggleRunAll,
    ToggleDebug(String),
    ToggleDebugAll,
    TriggerPendingAutomations,
    ToggleOutput(String),
    ToggleOutputAll
}
impl AsRef<Action> for Action {
    fn as_ref(&self) -> &Action {
        self
    }
}