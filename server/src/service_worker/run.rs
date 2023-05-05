use std::error::Error;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use reqwest::blocking::Client as HttpClient;
use reqwest::Method;

use shared::format_err;
use shared::message::models::{
    CompileStatus, HealthCheck, HttpMethod, OutputKey, OutputKind, RunStatus, ServiceAction,
};

use crate::service_worker::utils::{create_cmd, OnFinishParams, ProcessHandler};
use crate::ServerState;

pub fn handle_running(server_arc: Arc<Mutex<ServerState>>) -> Option<()> {
    let (mut command, service_name) = {
        let mut server = server_arc.lock().unwrap();

        let (service_name, command, exec_display) = {
            let profile = server.get_state().current_profile.as_ref()?;
            let runnable = profile
                .services
                .iter()
                .filter(|service| service.run.is_some())
                // Only consider services whose run-step has all dependencies satisfied
                .filter(|service| {
                    service.run
                        .as_ref()
                        .unwrap()
                        .dependencies
                        .iter()
                        .all(|dep| server.is_satisfied(dep))
                })
                .find(|service| {
                    let status = server.get_service_status(&service.name).unwrap();

                    match (&status.compile_status, &status.run_status) {
                        (_, RunStatus::Running | RunStatus::Healthy) => false,
                        (CompileStatus::Failed | CompileStatus::Compiling(_), _) => false,
                        // Allow services that have been fully compiled
                        (CompileStatus::PartiallyCompiled(_), _) => false,
                        (CompileStatus::None, RunStatus::Stopped) => service.compile.is_none() && status.should_run,
                        (CompileStatus::None, RunStatus::Failed) => service.compile.is_none() && status.should_run && status.action == ServiceAction::Restart,
                        (CompileStatus::FullyCompiled, RunStatus::Stopped) => status.should_run,
                        (CompileStatus::FullyCompiled, RunStatus::Failed) => status.should_run && status.action == ServiceAction::Restart,
                    }
                })?;

            let run_config = runnable.run.as_ref().unwrap();
            let exec_entry = &run_config.command;
            let command = create_cmd(exec_entry, runnable.dir.as_ref());

            (runnable.name.clone(), command, format!("{exec_entry}"))
        };

        server.update_service_status(&service_name, |status| {
            status.run_status = RunStatus::Running;
            status.action = ServiceAction::None;
        });

        server.add_output(
            &OutputKey {
                name: OutputKey::CTL.into(),
                service_ref: service_name.clone(),
                kind: OutputKind::Run,
            },
            format!("Exec: {exec_display}"),
        );

        (command, service_name)
    };

    match command.spawn() {
        Ok(handle) => {
            let handle = Arc::new(Mutex::new(handle));

            let health_check_thread = {
                let handle = handle.clone();
                let server = server_arc.clone();
                let service_name = service_name.clone();

                thread::spawn(move || {
                    let health_checks = server
                        .lock()
                        .unwrap()
                        .get_service(&service_name)
                        .map(|service| service.run.as_ref())
                        .flatten()
                        .map(|run_conf| run_conf.health_checks.clone())
                        .unwrap_or(Vec::new());

                    let http_client = HttpClient::new();

                    loop {
                        // If the process handle has exited, then we should not perform any health checks
                        if handle.lock().unwrap().try_wait().unwrap_or(None).is_some() {
                            break;
                        }

                        let mut successful = true;

                        for check in &health_checks {
                            match check {
                                HealthCheck::Http {
                                    url,
                                    method,
                                    timeout_millis,
                                    status,
                                } => {
                                    let result = http_client
                                        .request(
                                            match method {
                                                HttpMethod::GET => Method::GET,
                                                HttpMethod::POST => Method::POST,
                                                HttpMethod::PUT => Method::PUT,
                                                HttpMethod::PATCH => Method::PATCH,
                                                HttpMethod::DELETE => Method::DELETE,
                                                HttpMethod::OPTIONS => Method::OPTIONS,
                                            },
                                            url,
                                        )
                                        .timeout(Duration::from_millis(*timeout_millis))
                                        .send();

                                    if let Ok(response) = result {
                                        let response_status: u16 = response.status().into();
                                        if response_status != *status {
                                            successful = false;
                                            break;
                                        }
                                    } else {
                                        successful = false;
                                        break;
                                    }
                                }
                                HealthCheck::Port { port } => {
                                    if TcpListener::bind(format!("127.0.0.1:{port}")).is_err() {
                                        successful = false;
                                        break;
                                    }
                                }
                            }
                        }

                        // If all checks successful, break out of the loop
                        if successful {
                            break;
                        }

                        // Sleep for some time before reattempting, so we don't hog resources
                        thread::sleep(Duration::from_millis(100));
                    }

                    // If the process handle has exited, then we should not update the process status even if the
                    // checks passed
                    let has_exited = handle.lock().unwrap().try_wait().unwrap_or(None).is_some();
                    if !has_exited {
                        server
                            .lock()
                            .unwrap()
                            .update_service_status(&service_name, |status| {
                                // If the service is still running, update its status to healthy
                                if matches!(status.run_status, RunStatus::Running) {
                                    status.run_status = RunStatus::Healthy;
                                }
                            });
                    }
                })
            };

            // Register the health check thread into active threads
            server_arc
                .lock()
                .unwrap()
                .active_threads
                .push(health_check_thread);

            ProcessHandler {
                server: server_arc.clone(),
                handle,
                service_name: service_name.clone(),
                output: OutputKind::Run,
                on_finish: move |OnFinishParams { server, service_name, success, killed }| {
                    let mut server = server.lock().unwrap();
                    // Mark the service as no longer running when it exits
                    // TODO message
                    server.update_state(move |state| {
                        if success || killed {
                            state
                                .service_statuses
                                .get_mut(service_name)
                                .unwrap()
                                .run_status = RunStatus::Stopped;
                        } else {
                            state
                                .service_statuses
                                .get_mut(service_name)
                                .unwrap()
                                .run_status = RunStatus::Failed;
                        }
                    });
                },
                exit_early: move |(server, service_name)| {
                    let server = server.lock().unwrap();
                    let status = &server
                        .get_state()
                        .service_statuses
                        .get(service_name)
                        .unwrap();
                    let deps_satisfied = server.get_service(service_name)
                        .as_ref()
                        .unwrap()
                        .run
                        .as_ref()
                        .unwrap()
                        .dependencies
                        .iter()
                        .all(|dep| server.is_satisfied(dep));

                    (status.action == ServiceAction::Restart && deps_satisfied) || !status.should_run
                },
            }
            .launch();
        }
        Err(error) => {
            let mut server = server_arc.lock().unwrap();
            server.update_state(|state| {
                state
                    .service_statuses
                    .get_mut(&service_name)
                    .unwrap()
                    .run_status = RunStatus::Failed;
            });

            server.add_output(
                &OutputKey {
                    name: OutputKey::CTL.into(),
                    service_ref: service_name,
                    kind: OutputKind::Run,
                },
                format_err!("Failed to spawn child process", error),
            );
        }
    }

    Some(())
}
