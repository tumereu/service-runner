use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::system_state::SystemState;

mod process_wrapper;
mod worker;
mod block_worker;

pub use process_wrapper::*;

pub fn start_service_worker(state: Arc<Mutex<SystemState>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while !state.lock().unwrap().should_exit {
            worker::work_services(state.clone());
            thread::sleep(Duration::from_millis(10))
        }
    })
}
