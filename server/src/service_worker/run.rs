use shared::message::Broadcast;
use crate::service_worker::utils::{create_cmd, ProcessHandler};
use crate::ServerState;
use shared::message::models::{CompileStatus, RunStatus, OutputKind, ServiceAction, OutputKey};
use shared::system_state::Status;
use std::sync::{Mutex, Arc};

pub fn handle_running(server_arc: Arc<Mutex<ServerState>>) -> Option<()> {
    let mut server = server_arc.lock().unwrap();

    let (service_name, mut command) = {
        let profile = server.system_state.current_profile.as_ref()?;
        let runnable = profile.services.iter()
            .filter(|service| service.run.is_some())
            .find(|service| {
                // TODO dependencies?
                let status = server.system_state.service_statuses.get(&service.name).unwrap();
                // TODO service action
                match (&status.compile_status, &status.run_status) {
                    (_, RunStatus::Running) => false,
                    // TODO allow services with no compilation step at all?
                    (CompileStatus::None | CompileStatus::Failed | CompileStatus::Compiling(_), _) => false,
                    // Allow services that have been fully compiled
                    (CompileStatus::Compiled(index), _) => *index == service.compile.len() - 1,
                }
            })?;

        let run_config = runnable.run.as_ref().unwrap();

        let command = create_cmd(&run_config.command, runnable.dir.as_ref());

        (runnable.name.clone(), command)
    };

    // TODO update service action
    match command.spawn() {
        Ok(handle) => {
            server.system_state.service_statuses.get_mut(&service_name).unwrap().run_status = RunStatus::Running;
            let broadcast = Broadcast::State(server.system_state.clone());
            server.broadcast_all(broadcast);

            ProcessHandler {
                server: server_arc.clone(),
                handle,
                service_name: service_name.clone(),
                output: OutputKind::Run,
                on_finish: move |(server, service_name, success)| {
                    let mut server = server.lock().unwrap();
                    // Mark the service as no longer running when it exits
                    // TODO message
                    if success {
                        server.system_state.service_statuses.get_mut(service_name).unwrap().run_status = RunStatus::Stopped;
                    } else {
                        server.system_state.service_statuses.get_mut(service_name).unwrap().run_status = RunStatus::Failed;
                    }
                    let broadcast = Broadcast::State(server.system_state.clone());
                    server.broadcast_all(broadcast);
                },
                exit_early: move |(server, service_name)| {
                    let server = server.lock().unwrap();

                    // TODO simple test -- stop service when all services are running
                    let status = &server.system_state.service_statuses.get(service_name).unwrap();

                    status.action == ServiceAction::Restart || status.action == ServiceAction::Stop
                }
            }.launch(&mut server);
        }
        Err(error) => {
            server.system_state.service_statuses.get_mut(&service_name).unwrap().run_status = RunStatus::Failed;
            let broadcast = Broadcast::State(server.system_state.clone());
            server.broadcast_all(broadcast);

            server.add_output(&OutputKey {
                name: OutputKey::CTRL.into(),
                service_ref: service_name,
                kind: OutputKind::Compile,
            }, format!("Error in child process: {error}"));
        }
    }

    Some(())
}
