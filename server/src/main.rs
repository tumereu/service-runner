extern crate core;

use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{env, thread};

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
        start_action_processor(server.clone()),
        start_service_worker(server.clone()),
        start_file_watcher(server.clone()),
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

                    server.active_threads.retain(|thread| !thread.is_finished());
                    if Instant::now().duration_since(last_print).as_millis() >= 5000 {
                        let thread_count = server.active_threads.len();
                        dbg_println!("Active thread count: {thread_count}");
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
