mod utils;
mod compilation;
mod run;

use shared::system_state::Status;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use shared::dbg_println;
use shared::system_state::Status::Idle;
use crate::server_state::{ServerState};

pub fn start_service_worker(server: Arc<Mutex<ServerState>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while server.lock().unwrap().get_state().status != Status::Exiting {
            work_services(server.clone());
            thread::sleep(Duration::from_millis(1))
        }
    })
}

fn work_services(server: Arc<Mutex<ServerState>>) {
    compilation::handle_compilation(server.clone());
    run::handle_running(server.clone());
}
