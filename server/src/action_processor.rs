use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use shared::message::{Action, Broadcast};
use shared::message::models::ServiceStatus;
use shared::system_state::Status;

use crate::server_state::ServerState;

pub fn start_action_processor(server: Arc<Mutex<ServerState>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while server.lock().unwrap().system_state.status != Status::Exiting {
            {
                let mut server = server.lock().unwrap();
                while let Some(action) = server.actions_in.pop() {
                    process_action(&mut server, action);
                }
            }

            thread::sleep(Duration::from_millis(1))
        }
    })
}

fn process_action(
    server: &mut ServerState,
    action: Action
) {
    match action {
        Action::Shutdown => {
            server.system_state.status = Status::Exiting;
        }
        Action::ActivateProfile(profile) => {
            server.system_state.service_statuses = profile.services.iter()
                .map(|service| {
                    (service.name().clone(), ServiceStatus::from(&profile, service))
                }).collect();
            server.system_state.current_profile = Some(profile);
            let broadcast = Broadcast::State(server.system_state.clone());
            server.broadcast_all(broadcast);
        }
    }
}