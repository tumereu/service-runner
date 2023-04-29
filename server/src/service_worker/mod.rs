use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use shared::system_state::Status;

use crate::server_state::ServerState;

mod compilation;
mod run;
mod utils;

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
