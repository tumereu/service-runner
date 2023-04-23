use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use shared::message::Broadcast;

use crate::client_state::{ClientState, ClientStatus};
use crate::ui::UIState;

pub fn start_broadcast_processor(state: Arc<Mutex<ClientState>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while state.lock().unwrap().status != ClientStatus::Exiting {
            {
                let mut state = state.lock().unwrap();
                while let Some(broadcast) = state.broadcasts_in.pop_front() {
                    process_broadcast(&mut state, broadcast);
                }
            }

            thread::sleep(Duration::from_millis(1))
        }
    })
}

fn process_broadcast(
    state: &mut ClientState,
    broadcast: Broadcast
) {
    match broadcast {
        Broadcast::State(system_state) => {
            match state.ui {
                UIState::Initializing => {
                    if system_state.current_profile.is_none() {
                        state.ui = UIState::profile_select();
                    } else {
                        state.ui = UIState::view_profile();
                    }
                }
                UIState::ProfileSelect { .. } => {
                    if system_state.current_profile.is_some() {
                        state.ui = UIState::view_profile();
                    }
                }
                UIState::ViewProfile { .. } => {
                    if system_state.current_profile.is_none() {
                        state.ui = UIState::profile_select();
                    }
                }
            }

            state.system_state = Some(system_state);
        },
        Broadcast::OutputSync(store) => {
            state.output_store = store;
        },
        Broadcast::OutputLine(key, line) => {
            state.output_store.add_output(&key, line.value);
        }
    }
}
