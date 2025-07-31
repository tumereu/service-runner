use crate::config::{Block, Dependency, RequiredStatus, WorkDefinition};
use crate::models::{BlockAction, BlockStatus, GetBlock, OutputKey, OutputKind, Service};
use crate::runner::service_worker::process_wrapper::{create_cmd, OnFinishParams, ProcessWrapper};
use crate::runner::service_worker::ProcessStatus;
use crate::system_state::SystemState;
use crate::utils::format_err;
use itertools::Itertools;
use log::debug;
use std::sync::{Arc, Mutex};

pub fn work_services(state_arc: Arc<Mutex<SystemState>>) {
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
            work_block(state_arc.clone(), &service_id, &block_id);
        });
}

fn work_block(state_arc: Arc<Mutex<SystemState>>, service_id: &str, block_id: &str) {
    let work = {
        let state = state_arc.lock().unwrap();

        state
            .get_service(&service_id)
            .unwrap()
            .get_block(&block_id)
            .unwrap()
            .work
            .clone()
    };

    let prerequisites_satisfied = {
        let state = state_arc.lock().unwrap();
        let service = state.get_service(&service_id).unwrap();
        let block = service.get_block(&block_id).unwrap();

        block
            .prerequisites
            .iter()
            .all(|prerequisite| is_prerequisite_satisfied(&state, service, prerequisite))
    };

    let (status, action) = {
        let state = state_arc.lock().unwrap();
        let service = state.get_service(&service_id).unwrap();

        (
            service.get_block_status(&block_id).clone(),
            service.get_block_action(&block_id).clone(),
        )
    };

    let steps_to_complete = match work {
        WorkDefinition::CommandSeq { commands } => commands.len(),
        WorkDefinition::Process { .. } => 1,
    };

    // TODO check dependencies?
    match (status, action) {
        (
            BlockStatus::Working {
                steps_completed, ..
            },
            None,
        ) if steps_completed == steps_to_complete => {
            // TODO health check
            debug!("{service_id}.{block_id} is completed, updating status to OK");
            update_status(state_arc.clone(), service_id, block_id, BlockStatus::Ok)
        }

        (
            BlockStatus::Working {
                current_step,
                steps_completed,
            },
            None,
        ) if current_step.is_none() => {
            debug!("Block {service_id}.{block_id} is working (current step={current_step:?}, completed={steps_completed}). Executing next step");
            exec_next_work(state_arc.clone(), service_id, block_id, steps_completed);
        }

        (_, Some(BlockAction::ReRun)) => {
            let mut state = state_arc.lock().unwrap();

            match state.get_block_process(service_id, block_id) {
                Some(wrapper)
                    if wrapper.status.lock().unwrap().clone() == ProcessStatus::Running =>
                {
                    wrapper.stop()
                }
                Some(_) => {
                    state.set_block_process(service_id, block_id, None);
                }
                None => {
                    clear_current_action(state_arc.clone(), service_id, block_id);
                    update_status(
                        state_arc.clone(),
                        service_id,
                        block_id,
                        BlockStatus::Working {
                            steps_completed: 0,
                            current_step: None,
                        },
                    )
                }
            }
        }

        (BlockStatus::Initial | BlockStatus::Error, Some(BlockAction::Run)) => {
            clear_current_action(state_arc.clone(), service_id, block_id);

            // TODO perform health check
            update_status(
                state_arc.clone(),
                service_id,
                block_id,
                BlockStatus::Working {
                    steps_completed: 0,
                    current_step: None,
                },
            )
        }

        (BlockStatus::Working { .. } | BlockStatus::Ok, Some(BlockAction::Run)) => {
            clear_current_action(state_arc.clone(), service_id, block_id);
            update_status(
                state_arc.clone(),
                service_id,
                block_id,
                BlockStatus::Working {
                    steps_completed: 0,
                    current_step: None,
                },
            )
        }

        (_, Some(BlockAction::Enable)) => {
            // FIXME implement
            clear_current_action(state_arc.clone(), service_id, block_id);
        }
        (_, Some(BlockAction::Disable)) => {
            // FIXME implement
            clear_current_action(state_arc.clone(), service_id, block_id);
        }
        (_, Some(BlockAction::Stop)) => {
            // FIXME implement
            clear_current_action(state_arc.clone(), service_id, block_id);
        }
        (_, Some(BlockAction::Cancel)) => {
            // FIXME implement
            clear_current_action(state_arc.clone(), service_id, block_id);
        }
        (
            BlockStatus::Working {
                current_step,
                steps_completed,
            },
            None,
        ) if current_step.is_some() => {
            let process_status = state_arc
                .lock()
                .unwrap()
                .get_block_process(service_id, block_id)
                .map(|wrapper| wrapper.status.lock().unwrap().clone());

            match process_status {
                Some(ProcessStatus::Running) => {
                    // Still running, do nothing
                }
                Some(ProcessStatus::Failed) => {
                    debug!("Block {service_id}.{block_id} has failed at step {steps_completed}. Updating status to Error");
                    update_status(state_arc.clone(), service_id, block_id, BlockStatus::Error)
                }
                Some(ProcessStatus::Ok) => {
                    debug!("Block {service_id}.{block_id} has completed step {steps_completed}. Updating status to next step");
                    update_status(
                        state_arc.clone(),
                        service_id,
                        block_id,
                        BlockStatus::Working {
                            current_step: None,
                            steps_completed: steps_completed + 1,
                        },
                    )
                }
                None => update_status(state_arc.clone(), service_id, block_id, BlockStatus::Error),
            }
        }

        (_, None) => {
            // Intentionally do nothing: we're either currently performing some work, or are in some
            // other state with no action to execute
        }
    }

    let steps_completed: usize = {
        let state = state_arc.lock().unwrap();
        match state
            .get_service(&service_id)
            .unwrap()
            .get_block_status(&block_id)
        {
            BlockStatus::Working {
                steps_completed, ..
            } => steps_completed,
            _ => 0,
        }
    };
}

fn clear_current_action(state_arc: Arc<Mutex<SystemState>>, service_id: &str, block_id: &str) {
    let mut state = state_arc.lock().unwrap();
    state.update_service(service_id, |service| {
        service.update_block_action(block_id, None)
    })
}

fn update_status(
    state_arc: Arc<Mutex<SystemState>>,
    service_id: &str,
    block_id: &str,
    status: BlockStatus,
) {
    let mut state = state_arc.lock().unwrap();
    state.update_service(service_id, |service| {
        service.update_block_status(block_id, status)
    });
}

fn is_prerequisite_satisfied(
    state: &SystemState,
    service: &Service,
    prerequisite: &Dependency,
) -> bool {
    let referred_service_name = prerequisite
        .service
        .as_ref()
        .unwrap_or(&service.definition.id);

    state
        .iter_services()
        // Find the service the prerequisite refers to
        .find(|service| &service.definition.id == referred_service_name)
        .map(|service| {
            // Check that the status is acceptable according to the required status of the prereq
            match service.get_block_status(&prerequisite.stage) {
                BlockStatus::Initial => prerequisite.status == RequiredStatus::Initial,
                BlockStatus::Working { .. } => prerequisite.status == RequiredStatus::Working,
                BlockStatus::Ok => prerequisite.status == RequiredStatus::Ok,
                BlockStatus::Error => prerequisite.status == RequiredStatus::Error,
            }
        })
        .unwrap_or(false)
}

fn exec_next_work(
    state_arc: Arc<Mutex<SystemState>>,
    service_id: &str,
    block_id: &str,
    steps_completed: usize,
) {
    let work = {
        let state = state_arc.lock().unwrap();

        state
            .get_service(&service_id)
            .unwrap()
            .get_block(&block_id)
            .unwrap()
            .work
            .clone()
    };

    match work {
        WorkDefinition::CommandSeq { commands } => {
            let next_command = &commands[steps_completed];
            let mut command = create_cmd(
                next_command,
                Some(
                    state_arc
                        .lock()
                        .unwrap()
                        .get_service(&service_id)
                        .unwrap()
                        .definition
                        .dir
                        .clone(),
                ),
            );

            {
                let mut state = state_arc.lock().unwrap();
                state.add_output(
                    &OutputKey {
                        name: OutputKey::CTL.into(),
                        service_ref: service_id.to_owned(),
                        kind: OutputKind::Compile,
                    },
                    format!("Exec: {next_command}"),
                );

                state.update_service(&service_id, |service| {
                    service.update_block_status(
                        &block_id,
                        BlockStatus::Working {
                            steps_completed,
                            current_step: Some(steps_completed),
                        },
                    )
                });
            }

            match command.spawn() {
                Ok(process_handle) => {
                    let wrapper = ProcessWrapper::handle(
                        state_arc.clone(),
                        process_handle,
                        service_id.to_owned(),
                        block_id.to_owned(),
                    );

                    let mut state = state_arc.lock().unwrap();
                    state.set_block_process(service_id, block_id, Some(wrapper));
                }
                Err(error) => {
                    let mut state = state_arc.lock().unwrap();
                    state.update_service(&service_id, |service| {
                        service.update_block_status(&block_id, BlockStatus::Error)
                    });

                    state.add_output(
                        &OutputKey {
                            name: OutputKey::CTL.into(),
                            service_ref: service_id.to_owned(),
                            kind: OutputKind::Compile,
                        },
                        format_err!("Failed to spawn child process", error),
                    );
                }
            }
        }
        WorkDefinition::Process { executable } => {
            // TODO handle
        }
    }
}
