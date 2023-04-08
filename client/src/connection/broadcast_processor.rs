use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use shared::message::{Action, Broadcast};
use shared::system_state::{SystemState};
use crate::client_state::{ClientState, ClientStatus};
use crate::ui::UIState;

pub fn start_broadcast_processor(state: Arc<Mutex<ClientState>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while state.lock().unwrap().status != ClientStatus::Exiting {
            {
                let mut state = state.lock().unwrap();
                while let Some(broadcast) = state.broadcasts_in.pop() {
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
            state.system_state = Some(system_state);

            match state.ui {
                UIState::Initializing => {
                    state.ui = UIState::ProfileSelect {
                        selected_idx: 0
                    }
                }
                UIState::ProfileSelect { .. } => {}
            }
        }
    }
}
