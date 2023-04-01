use shared::config::Config;

pub struct AppState {
    pub config: Config,
    pub phase: Phase
}

#[derive(PartialEq, Eq)]
pub enum Phase {
    Initializing,
    Exit
}