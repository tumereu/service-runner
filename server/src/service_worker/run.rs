use std::error::Error;
use std::fmt::format;
use shared::message::Broadcast;
use crate::service_worker::utils::{create_cmd, ProcessHandler};
use crate::ServerState;
use shared::message::models::{CompileStatus, RunStatus, OutputKind, ServiceAction, OutputKey, ServiceStatus};
use shared::system_state::Status;
use std::sync::{Mutex, Arc};
use std::thread;
use std::time::Duration;
use shared::{dbg_println, format_err};

pub fn handle_running(server_arc: Arc<Mutex<ServerState>>) -> Option<()> {
    let mut server = server_arc.lock().unwrap();

    let (service_name, mut command, exec_display) = {
        let profile = server.get_state().current_profile.as_ref()?;
        let runnable = profile.services.iter()
            .filter(|service| service.run.is_some())
            .find(|service| {
                // TODO dependencies?
                let status = server.get_state().service_statuses.get(&service.name).unwrap();
                match (&status.compile_status, &status.run_status) {
                    (_, RunStatus::Running | RunStatus::Healthy) => false,
                    (CompileStatus::None, _) => service.compile.is_none(),
                    (CompileStatus::Failed | CompileStatus::Compiling(_), _) => false,
                    // Allow services that have been fully compiled
                    (CompileStatus::PartiallyCompiled(_), _) => false,
                    (CompileStatus::FullyCompiled, RunStatus::Stopped | RunStatus::Failed) => match status.action {
                        ServiceAction::Restart => true,
                        ServiceAction::None | ServiceAction::Stop | ServiceAction::Recompile => false,
                    }
                }
            })?;

        let run_config = runnable.run.as_ref().unwrap();
        let exec_entry = &run_config.command;
        let command = create_cmd(exec_entry, runnable.dir.as_ref());

        (runnable.name.clone(), command, format!("{exec_entry}"))
    };

    server.add_output(&OutputKey {
        name: OutputKey::CTRL.into(),
        service_ref: service_name.clone(),
        kind: OutputKind::Compile,
    }, format!("Exec: {exec_display}"));

    match command.spawn() {
        Ok(handle) => {
            server.update_state(|state| {
                state.service_statuses.get_mut(&service_name).unwrap().run_status = RunStatus::Running;
                state.service_statuses.get_mut(&service_name).unwrap().action = ServiceAction::None;
            });

            let handle = Arc::new(Mutex::new(handle));

            let health_check_thread = {
                let handle = handle.clone();
                let server = server_arc.clone();
                let service_name = service_name.clone();

                thread::spawn(move || {
                    let health_checks = server.lock().unwrap().get_service(&service_name)
                        .map(|service| service.run.as_ref())
                        .flatten()
                        .map(|run_conf| run_conf.health_check.clone())
                        .unwrap_or(Vec::new());

                    loop {
                        // If the process handle has exited, then we should not perform any health checks
                        if handle.lock().unwrap().try_wait().unwrap_or(None).is_some() {
                            break;
                        }

                        let mut successful = true;

                        for check in &health_checks {
                            // TODO implement actual checks
                            thread::sleep(Duration::from_millis(1000))
                        }

                        // If all checks successful, break out of the loop
                        if successful {
                            break;
                        }

                        // Sleep for some time before reattempting, so we don't hog resource
                        thread::sleep(Duration::from_millis(100));
                    }

                    // If the process handle has exited, then we should not update the process status even if the
                    // checks passed
                    if handle.lock().unwrap().try_wait().unwrap_or(None).is_none() {
                        server.lock().unwrap().update_service_status(&service_name, |status| {
                            // If the service is still running, update its status to healthy
                            if matches!(status.run_status, RunStatus::Running) {
                                status.run_status = RunStatus::Healthy;
                            }
                        });
                    }
                })
            };
            server.active_threads.push(health_check_thread);

            ProcessHandler {
                server: server_arc.clone(),
                handle,
                service_name: service_name.clone(),
                output: OutputKind::Run,
                on_finish: move |(server, service_name, success)| {
                    let mut server = server.lock().unwrap();
                    // Mark the service as no longer running when it exits
                    // TODO message
                    server.update_state(move |state| {
                        if success {
                            state.service_statuses.get_mut(service_name).unwrap().run_status = RunStatus::Stopped;
                        } else {
                            state.service_statuses.get_mut(service_name).unwrap().run_status = RunStatus::Failed;
                        }
                    });
                },
                exit_early: move |(server, service_name)| {
                    let server = server.lock().unwrap();

                    let status = &server.get_state().service_statuses.get(service_name).unwrap();

                    for status in server.get_state().service_statuses.values() {
                        if status.action != ServiceAction::None {
                            dbg_println!("Haz status {stat:?} ", stat = status.action);
                        }
                    }

                    status.action == ServiceAction::Restart || status.action == ServiceAction::Stop
                },
            }.launch(&mut server);
        }
        Err(error) => {
            server.update_state(|state| {
                state.service_statuses.get_mut(&service_name).unwrap().run_status = RunStatus::Failed;
                state.service_statuses.get_mut(&service_name).unwrap().action = ServiceAction::None;
            });

            server.add_output(&OutputKey {
                name: OutputKey::CTRL.into(),
                service_ref: service_name,
                kind: OutputKind::Compile,
            }, format_err!("Failed to spawn child process", error));
        }
    }

    Some(())
}
