use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use shared::message::{Action, Broadcast};
use shared::message::models::ServiceStatus;
use shared::system_state::Status;

use crate::server_state::ServerState;

pub fn start_action_processor(state: Arc<Mutex<ServerState>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while state.lock().unwrap().system_state.status != Status::Exiting {
            {
                let mut state = state.lock().unwrap();
                while let Some(action) = state.actions_in.pop() {
                    process_action(&mut state, action);
                }
            }

            thread::sleep(Duration::from_millis(1))
        }
    })
}

fn process_action(
    state: &mut ServerState,
    action: Action
) {
    match action {
        Action::Shutdown => {
            state.system_state.status = Status::Exiting;
        }
        Action::ActivateProfile(profile) => {
            state.system_state.service_statuses = profile.services.iter()
                .map(|service| {
                    (service.name().clone(), ServiceStatus::from(&profile, service))
                }).collect();
            state.system_state.current_profile = Some(profile);
            broadcast_state(state);
        }
    }
}

fn broadcast_state(state: &mut ServerState) {
    state.broadcasts_out.iter_mut().for_each(|(_key, value)| {
        value.push(Broadcast::State(state.system_state.clone()));
    });
}