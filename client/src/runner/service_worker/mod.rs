use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::system_state::SystemState;

mod compilation;
mod run;
mod utils;

pub fn start_service_worker(state: Arc<Mutex<SystemState>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        // TODO signals or something?
        while !state.lock().unwrap().should_exit {
            work_services(state.clone());
            thread::sleep(Duration::from_millis(10))
        }
    })
}

fn work_services(state: Arc<Mutex<SystemState>>) {
    compilation::handle_compilation(state.clone());
    run::handle_running(state.clone());
}
