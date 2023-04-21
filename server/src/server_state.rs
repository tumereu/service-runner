use std::collections::HashMap;
use std::thread::JoinHandle;
use std::time::Instant;

use shared::message::{Action, Broadcast};
use shared::message::models::{OutputKey, OutputStore, Service, ServiceStatus};
use shared::system_state::SystemState;

pub struct ServerState {
    pub created: Instant,
    pub actions_in: Vec<Action>,
    pub broadcasts_out: HashMap<u32, Vec<Broadcast>>,
    system_state: SystemState,
    pub output_store: OutputStore,
    pub active_threads: Vec<JoinHandle<()>>
}
impl ServerState {
    pub fn new() -> ServerState {
        ServerState {
            created: Instant::now(),
            actions_in: Vec::new(),
            broadcasts_out: HashMap::new(),
            system_state: SystemState::new(),
            output_store: OutputStore::new(),
            active_threads: Vec::new(),
        }
    }

    pub fn get_state(&self) -> &SystemState {
        &self.system_state
    }

    pub fn get_service(&self, service_name: &str) -> Option<&Service> {
        self.system_state.current_profile.as_ref()
            .map(|profile| profile.services.iter().find(|service| service.name == service_name))
            .flatten()
    }

    pub fn get_service_status(&self, service_name: &str) -> Option<&ServiceStatus> {
        self.system_state.service_statuses.get(service_name)
    }

    pub fn update_state<F>(&mut self, update: F) where F: FnOnce(&mut SystemState) {
        update(&mut self.system_state);
        let broadcast = Broadcast::State(self.system_state.clone());
        self.broadcast_all(broadcast);
    }

    pub fn update_service_status<F>(&mut self, service: &str, update: F) where F: FnOnce(&mut ServiceStatus) {
        self.update_state(move |state| {
            let mut status = state.service_statuses.get_mut(service).unwrap();
            update(status);
        });
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