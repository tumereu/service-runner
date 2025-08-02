use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::system_state::SystemState;

mod process_wrapper;
mod work_handler;
mod block_worker;
mod block_processor;
mod req_checker;

pub use process_wrapper::*;
use crate::runner::service_worker::block_processor::BlockProcessor;
use crate::runner::service_worker::block_worker::BlockWorker;

pub fn start_service_worker(state: Arc<Mutex<SystemState>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while !state.lock().unwrap().should_exit {
            work_services(state.clone());
            thread::sleep(Duration::from_millis(10))
        }
    })
}

fn work_services(state_arc: Arc<Mutex<SystemState>>) {
    // A collection of (service_id, block_id) pairs describing all services and their blocks
    // that might need to be worked on.
    let stages_to_work = {
        let state = state_arc.lock().unwrap();

        state
            .iter_services()
            .flat_map(|service| {
                service
                    .definition
                    .blocks
                    .iter()
                    .map(|block| (service.definition.id.clone(), block.id.clone()))
            })
            .collect::<Vec<_>>()
    };

    // Loop through all information we collected previously and launch appropriate subprocesses to
    // work them.
    stages_to_work
        .into_iter()
        .for_each(|(service_id, block_id)| {
            BlockWorker::new(
                state_arc.clone(),
                service_id,
                block_id,
            ).process_block();
        });
}
