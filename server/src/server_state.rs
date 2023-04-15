use std::collections::HashMap;
use std::process::Child;

use shared::message::{Action, Broadcast};
use shared::message::models::OutputStore;
use shared::system_state::SystemState;

pub struct ServerState {
    pub actions_in: Vec<Action>,
    pub broadcasts_out: HashMap<u32, Vec<Broadcast>>,
    pub system_state: SystemState,
    pub active_compile_count: usize,
    pub output_store: OutputStore
}
impl ServerState {
    pub fn new() -> ServerState {
        ServerState {
            actions_in: Vec::new(),
            broadcasts_out: HashMap::new(),
            system_state: SystemState::new(),
            active_compile_count: 0,
            output_store: OutputStore::new(),
        }
    }

    pub fn broadcast_all(&mut self, broadcast: Broadcast) {
        self.broadcasts_out.iter_mut().for_each(|(_key, value)| {
            value.push(broadcast.clone());
        });
    }

    pub fn broadcast_one(&mut self, client: u32, broadcast: Broadcast) {
        let mut queue = self.broadcasts_out.get_mut(&client);
        if let Some(queue) = queue {
            queue.push(broadcast);
        }
    }
}