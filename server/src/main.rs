extern crate core;

use std::{env, thread};
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use shared::dbg_println;
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

    let server = Arc::new(Mutex::new(ServerState::new()));

    start_action_processor(server.clone());
    start_service_worker(server.clone());

    let join_threads = {
        let server = server.clone();
        thread::spawn(move || {
            let mut last_print = Instant::now();

            loop {
                {
                    let mut server = server.lock().unwrap();
                    if server.get_state().status == Status::Exiting && server.active_threads.len() == 0 {
                        break;
                    }

                    server.active_threads.retain(|thread| !thread.is_finished());
                    if Instant::now().duration_since(last_print).as_millis() >= 5000 {
                        let thread_count = server.active_threads.len();
                        dbg_println!("Unjoined thread count: {thread_count}");
                        last_print = Instant::now();
                    }
                }

                thread::sleep(Duration::from_millis(10));
            }
        })
    };

    run_server(port, server.clone());
    join_threads.join().unwrap();

    Ok(())
}