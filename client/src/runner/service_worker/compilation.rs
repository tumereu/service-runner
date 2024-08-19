use std::sync::{Arc, Mutex};
use std::time::{Instant};
use crate::models::{AutomationTrigger, CompileStatus, OutputKey, OutputKind, AutomationEntry, ServiceAction};
use crate::models::AutomationEffect::Recompile;
use crate::runner::automation::{enqueue_automation, process_pending_automations};
use crate::runner::service_worker::utils::{create_cmd, OnFinishParams, ProcessHandler};
use crate::system_state::SystemState;
use crate::utils::format_err;

pub fn handle_compilation(state_arc: Arc<Mutex<SystemState>>) -> Option<()> {
    let (mut command, service_name, index) = {
        let mut state = state_arc.lock().unwrap();

        // Do not spawn new compilations if any are any currently active.
        if state
            .service_statuses
            .values()
            .any(|status| matches!(status.compile_status, CompileStatus::Compiling(_)))
        {
            return None;
        }

        let (service_name, command, exec_display, index) = {
            let profile = state.current_profile.as_ref()?;
            let (compilable, index) = profile
                .services
                .iter()
                .filter(|service| service.compile.is_some())
                // Only consider services whose compile step has all dependencies satisfied
                .filter(|service| {
                    service
                        .compile
                        .as_ref()
                        .unwrap()
                        .dependencies
                        .iter()
                        .all(|dep| state.is_satisfied(dep))
                })
                .flat_map(|service| {
                    let status = state
                        .service_statuses
                        .get(&service.name)
                        .unwrap();
                    match status.compile_status {
                        // Services currently compiling should not be compiled
                        CompileStatus::Compiling(_) => None,
                        // If we are not currently compiling, then a recompile requests means we should start again from
                        // the first step
                        _ if matches!(status.action, ServiceAction::Recompile) => {
                            Some((service, 0))
                        }
                        // Services with some but not all compile-steps should be compiled
                        CompileStatus::PartiallyCompiled(index) => Some((service, index + 1)),
                        // Fully compiled services do not need further compilation.
                        // Neither do failed or none-state services.
                        | CompileStatus::FullyCompiled
                        | CompileStatus::None
                        | CompileStatus::Failed => None,
                    }
                })
                .next()?;

            let exec_entry = compilable
                .compile
                .as_ref()
                .unwrap()
                .commands
                .get(index)
                .unwrap();
            let command = create_cmd(exec_entry, compilable.dir.as_ref());

            (
                compilable.name.clone(),
                command,
                format!("{exec_entry}"),
                index,
            )
        };

        state.add_output(
            &OutputKey {
                name: OutputKey::CTL.into(),
                service_ref: service_name.clone(),
                kind: OutputKind::Compile,
            },
            format!("Exec: {exec_display}"),
        );

        // Update the status of the service to be compiling and reset its action
        state.update_service_status(&service_name, |status| {
            status.compile_status = CompileStatus::Compiling(index);
            status.action = ServiceAction::None;
            // Remove any queued compile automations
            status.pending_automations.retain(|automation| {
                automation.effect != Recompile
            })
        });
        // Register the time that the compilation was started
        state.file_watchers.iter_mut()
            .for_each(|watcher_state| {
                watcher_state.latest_recompiles.insert(service_name.clone(), Instant::now());
            });

        (command, service_name, index)
    };

    match command.spawn() {
        Ok(handle) => {
            ProcessHandler {
                state: state_arc.clone(),
                handle: Arc::new(Mutex::new(handle)),
                service_name: service_name.clone(),
                output: OutputKind::Compile,
                exit_early: |_| false,
                on_finish: move |OnFinishParams { state: system_arc, service_name, success, exit_code, .. }| {
                    let mut system = system_arc.lock().unwrap();

                    if success {
                        let num_steps = system
                            .get_service(service_name)
                            .as_ref()
                            .map(|service| service.compile.as_ref())
                            .flatten()
                            .map(|compile| compile.commands.len())
                            .unwrap_or(0);

                        system.update_service_status(&service_name, move |status| {
                            status.compile_status = if index >= num_steps - 1 {
                                CompileStatus::FullyCompiled
                            } else {
                                CompileStatus::PartiallyCompiled(index)
                            };
                            status.action = ServiceAction::Restart;
                        });

                        // Process automation: if a service has an automation entry triggered by the compilation of this
                        // service, then we should queue a pending automation for that service
                        let automations_to_enqueue: Vec<(String, AutomationEntry)> = system.iter_services()
                            .flat_map(|service| {
                                service.automation
                                    .iter()
                                    .filter(|entry| {
                                        match &entry.trigger {
                                            AutomationTrigger::RecompiledService { service } => service == service_name,
                                            _ => false
                                        }
                                    }).map(|automation_entry| (service.name.clone(), automation_entry.clone()))
                            }).collect();
                        automations_to_enqueue.iter().for_each(|(service, entry)| {
                            enqueue_automation(&mut system, &service, &entry)
                        });

                        // Check the automations immediately, so that non-debounced automations fire without delay
                        process_pending_automations(&mut system);
                    } else {
                        system.update_service_status(&service_name, |status| {
                            status.compile_status = CompileStatus::Failed;
                            status.action = ServiceAction::None;
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
            let mut server = state_arc.lock().unwrap();
            server.update_service_status(&service_name, |status| {
                status.compile_status = CompileStatus::Failed;
            });
            server.add_output(
                &OutputKey {
                    name: OutputKey::CTL.into(),
                    service_ref: service_name,
                    kind: OutputKind::Compile,
                },
                format_err!("Failed to spawn child process", error),
            );
        }
    }

    Some(())
}
