use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use itertools::Itertools;
pub use concurrent_operation::*;
use crate::models::TaskStatus;
use crate::runner::service_worker::block_processor::BlockProcessor;
use crate::runner::service_worker::service_block_context::ServiceBlockContext;
use crate::system_state::SystemState;

mod concurrent_operation;
mod work_handler;
mod service_block_context;
mod block_processor;
mod requirement_checker;
mod work_context;
mod work_sequence_executor;
mod task_context;

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
    let blocks_to_work = {
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
    
    let tasks_to_work = {
        let state = state_arc.lock().unwrap();
        
        state.current_profile.iter().flat_map(|profile| profile.tasks.iter())
            .enumerate()
            .filter(|(_, task)| matches!(task.status, TaskStatus::Running { .. }))
            .map(|(idx, task)| (idx, task.definition_id.clone()))
            .collect::<Vec<_>>()
    };

    // Loop through all information we collected previously and launch appropriate subprocesses to
    // work them.
    blocks_to_work
        .into_iter()
        .for_each(|(service_id, block_id)| {
            ServiceBlockContext::new(
                state_arc.clone(),
                service_id,
                block_id,
            ).process_block();
        });
    
    tasks_to_work.into_iter().for_each(|task| {
        
    })
}
