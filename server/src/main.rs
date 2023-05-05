extern crate core;

use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{env, thread};
use itertools::Itertools;

use shared::dbg_println;
use shared::system_state::Status;

use crate::action_processor::start_action_processor;
use crate::connection::run_server;
use crate::file_watcher::start_file_watcher;
use crate::server_state::ServerState;
use crate::service_worker::start_service_worker;

mod action_processor;
mod connection;
mod server_state;
mod service_worker;
mod file_watcher;

fn main() -> Result<(), Box<dyn Error>> {
    let port: u16 = env::args()
        .collect::<Vec<String>>()
        .get(1)
        .unwrap()
        .parse()?;

    let server = Arc::new(Mutex::new(ServerState::new()));

    let mut handles = vec![
        ("action-processor".into(), start_action_processor(server.clone())),
        ("service-worker".into(), start_service_worker(server.clone())),
        ("file-watcher".into(), start_file_watcher(server.clone())),
    ];

    server.lock().unwrap().active_threads.append(&mut handles);

    let join_threads = {
        let server = server.clone();
        thread::spawn(move || {
            let mut last_print = Instant::now();

            loop {
                {
                    let mut server = server.lock().unwrap();
                    if server.get_state().status == Status::Exiting
                        && server.active_threads.len() == 0
                    {
                        break;
                    }

                    server.active_threads.retain(|(_, thread)| !thread.is_finished());

                    let print_delay = if server.get_state().status == Status::Exiting {
                        Duration::from_millis(1000)
                    } else {
                        Duration::from_millis(60_000)
                    };

                    if Instant::now().duration_since(last_print) >= print_delay {
                        let status = if server.get_state().status == Status::Exiting {
                            "Server is trying to exit"
                        } else {
                            "Server running normally"
                        };

                        let thread_count = server.active_threads.len();
                        let threads = server.active_threads.iter()
                            .map(|(name, _)| name)
                            .join(", ");
                        dbg_println!("{status}. Active threads ({thread_count} total): {threads}");
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
