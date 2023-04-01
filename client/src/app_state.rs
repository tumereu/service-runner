use model::config::ServiceConfig;

pub struct AppState {
    pub config: ServiceConfig,
    pub phase: Phase
}

#[derive(PartialEq, Eq)]
pub enum Phase {
    Initializing,
    Exit
}