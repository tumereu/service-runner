use std::collections::HashMap;
use std::thread::JoinHandle;

use shared::message::{Action, Broadcast};
use shared::message::models::{OutputKey, OutputStore};
use shared::system_state::SystemState;

pub struct ServerState {
    pub actions_in: Vec<Action>,
    pub broadcasts_out: HashMap<u32, Vec<Broadcast>>,
    pub system_state: SystemState,
    pub active_compile_count: usize,
    pub output_store: OutputStore,
    pub active_threads: Vec<JoinHandle<()>>
}
impl ServerState {
    pub fn new() -> ServerState {
        ServerState {
            actions_in: Vec::new(),
            broadcasts_out: HashMap::new(),
            system_state: SystemState::new(),
            active_compile_count: 0,
            output_store: OutputStore::new(),
            active_threads: Vec::new(),
        }
    }

    pub fn broadcast_all(&mut self, broadcast: Broadcast) {
        self.broadcasts_out.iter_mut().for_each(|(_key, value)| {
            value.push(broadcast.clone());
        });
    }

    pub fn broadcast_one(&mut self, client: u32, broadcast: Broadcast) {
        let queue = self.broadcasts_out.get_mut(&client);
        if let Some(queue) = queue {
            queue.push(broadcast);
        }
    }

    pub fn add_output(&mut self, key: &OutputKey, line: String) {
        let line = self.output_store.add_output(key, line).clone();
        self.broadcast_all(Broadcast::OutputLine(key.clone(), line));
    }
}