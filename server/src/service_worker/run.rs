use std::error::Error;
use shared::message::Broadcast;
use crate::service_worker::utils::{create_cmd, ProcessHandler};
use crate::ServerState;
use shared::message::models::{CompileStatus, RunStatus, OutputKind, ServiceAction, OutputKey};
use shared::system_state::Status;
use std::sync::{Mutex, Arc};
use shared::format_err;

pub fn handle_running(server_arc: Arc<Mutex<ServerState>>) -> Option<()> {
    let mut server = server_arc.lock().unwrap();

    let (service_name, mut command) = {
        let profile = server.get_state().current_profile.as_ref()?;
        let runnable = profile.services.iter()
            .filter(|service| service.run.is_some())
            .find(|service| {
                // TODO dependencies?
                let status = server.get_state().service_statuses.get(&service.name).unwrap();
                // TODO service action
                match (&status.compile_status, &status.run_status) {
                    (_, RunStatus::Running) => false,
                    // TODO allow services with no compilation step at all?
                    (CompileStatus::None | CompileStatus::Failed | CompileStatus::Compiling(_), _) => false,
                    // Allow services that have been fully compiled
                    (CompileStatus::Compiled(index), _) if *index < service.compile.len() - 1 => false,
                    (CompileStatus::Compiled(_), _) => match status.action {
                        ServiceAction::Restart => true,
                        ServiceAction::None | ServiceAction::Stop | ServiceAction::Recompile => false,
                    }
                }
            })?;

        let run_config = runnable.run.as_ref().unwrap();

        let command = create_cmd(&run_config.command, runnable.dir.as_ref());

        (runnable.name.clone(), command)
    };

    // TODO update service action
    // TODO ctrl output
    match command.spawn() {
        Ok(handle) => {
            server.update_state(|state| {
                state.service_statuses.get_mut(&service_name).unwrap().run_status = RunStatus::Running;
                state.service_statuses.get_mut(&service_name).unwrap().action = ServiceAction::None;
            });

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

                    status.action == ServiceAction::Restart || status.action == ServiceAction::Stop
                }
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
