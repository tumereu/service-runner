use crate::config::{Dependency, RequiredStatus, Stage, StageWork};
use crate::models::{GetStage, OutputKey, OutputKind, Service, StageStatus};
use crate::system_state::SystemState;
use std::sync::{Arc, Mutex};
use itertools::Itertools;
use crate::runner::service_worker::utils::{create_cmd, OnFinishParams, ProcessHandler};
use crate::utils::format_err;

pub fn work_services(state_arc: Arc<Mutex<SystemState>>) -> Option<()> {
    // A collection of (service_name, stage_name) pairs describing the services and their stages
    // that should be worked on.
    let stages_to_work = {
        let state = state_arc.lock().unwrap();

        state.iter_services().flat_map(|service| {
            service
                .definition
                .stages
                .iter()
                .filter(|stage| {
                    // All prerequisites for the stage must be met
                    stage
                        .prerequisites
                        .iter()
                        .all(|prerequisite| is_prerequisite_satisfied(&state, service, prerequisite))
                })
                .filter(|stage| {
                    // The stage must be in a runnable state and not already executing some work
                    match service.get_stage_status(&stage.name) {
                        StageStatus::Initial => true,
                        StageStatus::Working { current_step, .. } => current_step.is_none(),
                        StageStatus::Ok => false,
                        StageStatus::Error => false,
                    }
                })
                .map(|stage| {
                    (service.definition.name.clone(), stage.name.clone())
                })
        }).collect::<Vec<_>>()
    };

    // Loop through all the stages we collected previously and launch appropriate subprocesses to
    // work them.
    stages_to_work.into_iter().for_each(|(service_name, stage_name)| {
        work_stage(state_arc.clone(), service_name, stage_name);
    });

    Some(())
}

fn work_stage(state_arc: Arc<Mutex<SystemState>>, service_name: String, stage_name: String) {
    let work = {
        let state = state_arc.lock().unwrap();

        state.get_service(&service_name)
            .unwrap()
            .get_stage(&stage_name)
            .unwrap()
            .work.clone()
    };

    let steps_completed: usize = {
        let state = state_arc.lock().unwrap();
        match state.get_service(&service_name)
            .unwrap().get_stage_status(&stage_name) {
            StageStatus::Working { steps_completed, .. }  => steps_completed,
            _ => 0
        }
    };

    match work {
        StageWork::CommandSeq { commands } => {
            let next_command = &commands[steps_completed];
            let mut command = create_cmd(
                next_command,
                Some(state_arc.lock().unwrap().get_service(&service_name).unwrap().definition.dir.clone())
            );

            {
                let mut state = state_arc.lock().unwrap();
                state.add_output(
                    &OutputKey {
                        name: OutputKey::CTL.into(),
                        service_ref: service_name.clone(),
                        kind: OutputKind::Compile,
                    },
                    format!("Exec: {next_command}"),
                );

                state.update_service(&service_name, |service| {
                    service.update_stage_status(&stage_name, StageStatus::Working {
                        steps_completed,
                        current_step: Some(steps_completed)
                    })
                });
            }

            match command.spawn() {
                Ok(handle) => {
                    ProcessHandler {
                        state: state_arc.clone(),
                        service_name: service_name.clone(),
                        handle: Arc::new(Mutex::new(handle)),
                        output: OutputKind::Compile,
                        exit_early: |_| false,
                        on_finish: move |OnFinishParams { state: system_arc, success, exit_code, .. }| {
                            let mut system = system_arc.lock().unwrap();

                            if success {
                                system.update_service(&service_name, |service| {
                                    service.update_stage_status(
                                        &stage_name,
                                        if steps_completed + 1 == commands.len() {
                                            // TODO health checks?
                                            StageStatus::Ok
                                        } else {
                                            StageStatus::Working {
                                                steps_completed: steps_completed + 1,
                                                current_step: None,
                                            }
                                        }
                                    )
                                });

                                // TODO publish event or something here?
                                // TODO add output?
                            } else {
                                system.update_service(&service_name, |service| {
                                    service.update_stage_status(
                                        &stage_name,
                                        StageStatus::Error
                                    )
                                });

                                system.add_output(
                                    &OutputKey {
                                        name: OutputKey::CTL.into(),
                                        service_ref: service_name.into(),
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
                    state.update_service(&service_name, |service| {
                        service.update_stage_status(
                            &stage_name,
                            StageStatus::Error
                        )
                    });

                    state.add_output(
                        &OutputKey {
                            name: OutputKey::CTL.into(),
                            service_ref: service_name,
                            kind: OutputKind::Compile,
                        },
                        format_err!("Failed to spawn child process", error),
                    );
                }
            }
        }
        StageWork::Process { executable } => {
            // TODO handle
        }
    }
}

fn is_prerequisite_satisfied(state: &SystemState, service: &Service, prerequisite: &Dependency) -> bool {
    let referred_service_name = prerequisite
        .service
        .as_ref()
        .unwrap_or(&service.definition.name);

    state
        .iter_services()
        // Find the service the prerequisite refers to
        .find(|service| &service.definition.name == referred_service_name)
        .map(|service| {
            // Check that the status is acceptable according to the required status of the prereq
            match service.get_stage_status(&prerequisite.stage) {
                StageStatus::Initial => prerequisite.status == RequiredStatus::Initial,
                StageStatus::Working{ .. } => prerequisite.status == RequiredStatus::Working,
                StageStatus::Ok => prerequisite.status == RequiredStatus::Ok,
                StageStatus::Error => prerequisite.status == RequiredStatus::Error,
            }
        })
        .unwrap_or(false)
}
