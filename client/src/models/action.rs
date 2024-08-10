use crate::models::{Profile, ServiceAction};

pub enum Action {
    Tick,
    Shutdown,
    ActivateProfile(Profile),
    UpdateServiceAction(String, ServiceAction),
    UpdateAllServiceActions(ServiceAction),
    CycleAutoCompile(String),
    CycleAutoCompileAll,
    ToggleRun(String),
    ToggleRunAll,
    ToggleDebug(String),
    ToggleDebugAll,
    TriggerPendingCompiles,
    ToggleOutput(String),
    ToggleOutputAll
}
impl AsRef<Action> for Action {
    fn as_ref(&self) -> &Action {
        self
    }
}