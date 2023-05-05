use std::sync::{Arc, Mutex};
use std::time::Instant;

use shared::format_err;
use shared::message::models::{CompileStatus, OutputKey, OutputKind, ServiceAction, AutoCompileTrigger, AutoCompileMode};

use crate::service_worker::utils::{create_cmd, OnFinishParams, ProcessHandler};
use crate::ServerState;

pub fn handle_compilation(server_arc: Arc<Mutex<ServerState>>) -> Option<()> {
    let (mut command, service_name, index) = {
        let mut server = server_arc.lock().unwrap();

        // Do not spawn new compilations if any are any currently active.
        if server
            .get_state()
            .service_statuses
            .values()
            .any(|status| matches!(status.compile_status, CompileStatus::Compiling(_)))
        {
            return None;
        }

        let (service_name, command, exec_display, index) = {
            let profile = server.get_state().current_profile.as_ref()?;
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
                        .all(|dep| server.is_satisfied(dep))
                })
                .flat_map(|service| {
                    let status = server
                        .get_state()
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

        server.add_output(
            &OutputKey {
                name: OutputKey::CTL.into(),
                service_ref: service_name.clone(),
                kind: OutputKind::Compile,
            },
            format!("Exec: {exec_display}"),
        );

        // Update the status of the service to be compiling and reset its action
        server.update_service_status(&service_name, |status| {
            status.compile_status = CompileStatus::Compiling(index);
            status.action = ServiceAction::None;
            status.has_uncompiled_changes = false;
        });
        // Register the time that the compilation was started
        server.file_watchers.iter_mut()
            .for_each(|watcher_state| {
                watcher_state.latest_recompiles.insert(service_name.clone(), Instant::now());
            });

        (command, service_name, index)
    };

    match command.spawn() {
        Ok(handle) => {
            ProcessHandler {
                server: server_arc.clone(),
                handle: Arc::new(Mutex::new(handle)),
                service_name: service_name.clone(),
                output: OutputKind::Compile,
                exit_early: |_| false,
                on_finish: move |OnFinishParams { server, service_name, success, exit_code, .. }| {
                    let mut server = server.lock().unwrap();
                    if success {
                        let num_steps = server
                            .get_service(service_name)
                            .as_ref()
                            .map(|service| service.compile.as_ref())
                            .flatten()
                            .map(|compile| compile.commands.len())
                            .unwrap_or(0);

                        server.update_service_status(&service_name, move |status| {
                            status.compile_status = if index >= num_steps - 1 {
                                CompileStatus::FullyCompiled
                            } else {
                                CompileStatus::PartiallyCompiled(index)
                            };
                            status.action = ServiceAction::Restart;
                        });

                        let triggered_recompiles: Vec<String> = server.iter_services()
                            .filter(|service| {
                                // The service matches if it has an autocompile-entry that references the compilation
                                // of the just-now compiled service
                                service.autocompile
                                    .iter()
                                    .flat_map(|autocompile| &autocompile.triggers)
                                    .any(|trigger| {
                                        match trigger {
                                            AutoCompileTrigger::RecompiledService { service } => service == service_name,
                                            _ => false
                                        }
                                    })
                            })
                            .map(|service| service.name.clone())
                            .collect();

                        // Trigger recompile for any service that has the just-compiled service as a trigger
                        triggered_recompiles
                            .iter()
                            .for_each(|service| {
                                server.update_service_status(&service, |status| {
                                    match status.auto_compile {
                                        Some(AutoCompileMode::AUTOMATIC) => {
                                            status.action = ServiceAction::Recompile;
                                        },
                                        Some(AutoCompileMode::TRIGGERED) => {
                                            status.has_uncompiled_changes = true;
                                        },
                                        _ => {}
                                    }
                                })
                            });
                    } else {
                        server.update_service_status(&service_name, |status| {
                            status.compile_status = CompileStatus::Failed;
                            status.action = ServiceAction::None;
                        });

                        server.add_output(
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
            let mut server = server_arc.lock().unwrap();
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
