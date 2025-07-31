use crate::config::{Dependency, RequiredStatus, Block, WorkDefinition};
use crate::models::{GetBlock, OutputKey, OutputKind, Service, BlockStatus, BlockAction};
use crate::system_state::SystemState;
use std::sync::{Arc, Mutex};
use itertools::Itertools;
use crate::runner::service_worker::utils::{create_cmd, OnFinishParams, ProcessHandler};
use crate::utils::format_err;

pub fn work_services(state_arc: Arc<Mutex<SystemState>>) {
    // A collection of (service_name, stage_name) pairs describing the services and their stages
    // that should be worked on.
    let stages_to_work = {
        let state = state_arc.lock().unwrap();

        state.iter_services().flat_map(|service| {
            service
                .definition
                .blocks
                .iter()
                .filter(|block| {
                    // All prerequisites for the stage must be met
                    block
                        .prerequisites
                        .iter()
                        .all(|prerequisite| is_prerequisite_satisfied(&state, service, prerequisite))
                })
                .filter(|block| {
                    match service.get_block_action(&block.id) {
                        None => match service.get_block_status(&block.id) {
                            BlockStatus::Initial => false,
                            BlockStatus::Working { current_step, .. } => current_step.is_none(),
                            BlockStatus::Ok => false,
                            BlockStatus::Error => false,
                        },
                        Some(BlockAction::Run) => match service.get_block_status(&block.id) {
                            BlockStatus::Initial => true,
                            BlockStatus::Working { current_step, .. } => current_step.is_none(),
                            BlockStatus::Ok => false,
                            BlockStatus::Error => false,
                        }
                        Some(BlockAction::ReRun) => true,
                        // TODO do something with these?
                        Some(BlockAction::Enable) => false,
                        Some(BlockAction::Disable) => false,
                        Some(BlockAction::Cancel) => false,
                        Some(BlockAction::Stop) => false,
                    }
                })
                .map(|stage| {
                    (service.definition.id.clone(), stage.id.clone())
                })
        }).collect::<Vec<_>>()
    };

    // Loop through all the stages we collected previously and launch appropriate subprocesses to
    // work them.
    stages_to_work.into_iter().for_each(|(service_name, stage_name)| {
        work_block(state_arc.clone(), &service_name, &stage_name);
    });
}

fn work_block(state_arc: Arc<Mutex<SystemState>>, service_id: &str, block_id: &str) {
    let work = {
        let state = state_arc.lock().unwrap();

        state.get_service(&service_id)
            .unwrap()
            .get_block(&block_id)
            .unwrap()
            .work.clone()
    };

    let prerequisites_satisfied = {
        let state = state_arc.lock().unwrap();
        let service = state.get_service(&service_id).unwrap();
        let block = service.get_block(&block_id).unwrap();

        block.prerequisites
            .iter()
            .all(|prerequisite| is_prerequisite_satisfied(&state, service, prerequisite))
    };

    let (status, action) = {
        let state = state_arc.lock().unwrap();
        let service = state.get_service(&service_id).unwrap();

        (service.get_block_status(&block_id).clone(), service.get_block_action(&block_id).clone())
    };

    match (status, action) {
        (BlockStatus::Working { current_step, ..}, None) if current_step.is_none() => {
            // TODO exec next
        }
        (BlockStatus::Initial | BlockStatus::Error, Some(BlockAction::Run) | Some(BlockAction::ReRun)) => {
            // TODO start work
        }
        (BlockStatus::Working { .. } | BlockStatus::Ok, Some(BlockAction::Run)) => {
            clear_current_action(state_arc.clone(), service_id, block_id);
        }
        (BlockStatus::Working { .. }, Some(BlockAction::ReRun)) => {
            // TODO stop current work, start new
        }
        (BlockStatus::Ok, Some(BlockAction::ReRun)) => {
            // TODO stop current process if eligible, then rerun
        }

        (_, Some(BlockAction::Enable)) => {
            // FIXME implement
        }
        (_, Some(BlockAction::Disable)) => {
            // FIXME implement
        }
        (_, Some(BlockAction::Stop)) => {
            // FIXME implement
        }
        (_, Some(BlockAction::Cancel)) => {
            // FIXME implement
        }
        (BlockStatus::Working { current_step, ..}, None) if current_step.is_some() => {
            // TODO check work status
        }
        
        (_, None) => {
            // Intentionally do nothing: we're either currently performing some work, or are in some
            // other state with no action to execute
        }
    }

    let steps_completed: usize = {
        let state = state_arc.lock().unwrap();
        match state.get_service(&service_id)
            .unwrap().get_block_status(&block_id) {
            BlockStatus::Working { steps_completed, .. }  => steps_completed,
            _ => 0
        }
    };



    match work {
        WorkDefinition::CommandSeq { commands } => {
            let next_command = &commands[steps_completed];
            let mut command = create_cmd(
                next_command,
                Some(state_arc.lock().unwrap().get_service(&service_id).unwrap().definition.dir.clone())
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
                    service.update_block_status(&block_id, BlockStatus::Working {
                        steps_completed,
                        current_step: Some(steps_completed)
                    })
                });
            }

            match command.spawn() {
                Ok(handle) => {
                    ProcessHandler {
                        state: state_arc.clone(),
                        thread_prefix: service_id.to_owned(),
                        handle: Arc::new(Mutex::new(handle)),
                        output: OutputKind::Compile,
                        exit_early: |_| false,
                        on_finish: move |OnFinishParams { state: system_arc, success, exit_code, .. }| {
                            let mut system = system_arc.lock().unwrap();

                            if success {
                                system.update_service(&service_id, |service| {
                                    service.update_block_status(
                                        &block_id,
                                        if steps_completed + 1 == commands.len() {
                                            // TODO health checks?
                                            BlockStatus::Ok
                                        } else {
                                            BlockStatus::Working {
                                                steps_completed: steps_completed + 1,
                                                current_step: None,
                                            }
                                        }
                                    )
                                });

                                // TODO publish event or something here?
                                // TODO add output?
                            } else {
                                system.update_service(&service_id, |service| {
                                    service.update_block_status(
                                        &block_id,
                                        BlockStatus::Error
                                    )
                                });

                                system.add_output(
                                    &OutputKey {
                                        name: OutputKey::CTL.into(),
                                        service_ref: service_id.into(),
                                        kind: OutputKind::Compile,
                                    },
                                    format!("Process exited with a non-zero status code {exit_code}"),
                                );
                            }
                        },
                    }
                        .launch();
                }
                Err(error) => {
                    let mut state = state_arc.lock().unwrap();
                    state.update_service(&service_id, |service| {
                        service.update_block_status(
                            &block_id,
                            BlockStatus::Error
                        )
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

fn clear_current_action(state_arc: Arc<Mutex<SystemState>>, service_id: &str, block_id: &str) {
    let mut state = state_arc.lock().unwrap();
    state.update_service(service_id, |service| service.update_block_action(block_id, None))
}

fn is_prerequisite_satisfied(state: &SystemState, service: &Service, prerequisite: &Dependency) -> bool {
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
                BlockStatus::Working{ .. } => prerequisite.status == RequiredStatus::Working,
                BlockStatus::Ok => prerequisite.status == RequiredStatus::Ok,
                BlockStatus::Error => prerequisite.status == RequiredStatus::Error,
            }
        })
        .unwrap_or(false)
}
