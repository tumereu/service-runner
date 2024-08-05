use crate::model::message::models::{OutputKey, OutputLine, OutputStore, Profile, ServiceAction};
use crate::model::system_state::SystemState;

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

#[derive(Clone, Debug)]
pub enum Broadcast {
    State(SystemState),
    OutputLine(OutputKey, OutputLine),
    OutputSync(OutputStore),
}
impl AsRef<Broadcast> for Broadcast {
    fn as_ref(&self) -> &Broadcast {
        self
    }
}
