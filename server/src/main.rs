extern crate core;

use std::{env, thread};
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use shared::system_state::Status;

use crate::action_processor::start_action_processor;
use crate::connection::run_server;
use crate::server_state::ServerState;
use crate::service_worker::start_service_worker;

mod server_state;
mod connection;
mod action_processor;
mod service_worker;

fn main() -> Result<(), Box<dyn Error>> {
    let port: u16 = env::args().collect::<Vec<String>>()
        .get(1)
        .unwrap()
        .parse()?;

    let state = Arc::new(Mutex::new(ServerState::new()));

    start_action_processor(state.clone());
    start_service_worker(state.clone());

    let join_threads = {
        let state = state.clone();
        thread::spawn(move || {
            loop {
                {
                    let mut state = state.lock().unwrap();
                    if state.system_state.status == Status::Exiting && state.active_threads.len() == 0 {
                        break;
                    }

                    state.active_threads.retain(|thread| !thread.is_finished());
                }

                thread::sleep(Duration::from_millis(10));
            }
        })
    };

    run_server(port, state.clone());
    join_threads.join().unwrap();

    Ok(())
}