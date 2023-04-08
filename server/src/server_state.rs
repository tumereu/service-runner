use std::collections::HashMap;

use shared::message::{Action, Broadcast};
use shared::system_state::SystemState;

pub struct ServerState {
    pub actions_in: Vec<Action>,
    pub broadcasts_out: HashMap<u32, Vec<Broadcast>>,
    pub system_state: SystemState
}
impl ServerState {
    pub fn new() -> ServerState {
        ServerState {
            actions_in: Vec::new(),
            broadcasts_out: HashMap::new(),
            system_state: SystemState::new()
        }
    }
}