use std::collections::HashMap;
use std::process::Child;

use shared::message::{Action, Broadcast};
use shared::system_state::SystemState;

pub struct ServerState {
    pub actions_in: Vec<Action>,
    pub broadcasts_out: HashMap<u32, Vec<Broadcast>>,
    pub system_state: SystemState,
    pub active_compile_count: usize,
}
impl ServerState {
    pub fn new() -> ServerState {
        ServerState {
            actions_in: Vec::new(),
            broadcasts_out: HashMap::new(),
            system_state: SystemState::new(),
            active_compile_count: 0
        }
    }
}

#[derive(Debug)]
pub struct Process {
    pub handle: Child,
    pub service: String,
    pub index: usize,
}