use shared::message::Broadcast;
use crate::service_worker::utils::{spawn_handler, create_cmd};
use crate::ServerState;
use shared::message::models::{CompileStatus, OutputKind};
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
                match status.compile_status {
                    // TODO handle restarts?
                    _ if status.is_running => false,
                    // TODO allow services with no compilation step at all?
                    CompileStatus::None | CompileStatus::Compiling(_) => false,
                    // Allow services that have been fully compiled
                    CompileStatus::Compiled(index) => index == service.compile.len() - 1,
                }
            })?;

        let run_config = runnable.run.as_ref().unwrap();

        let mut command = create_cmd(&run_config.command, runnable.dir.as_ref());

        (runnable.name.clone(), command)
    };

    server.system_state.service_statuses.get_mut(&service_name).unwrap().is_running = true;
    let broadcast = Broadcast::State(server.system_state.clone());
    server.broadcast_all(broadcast);

    let handle = command.spawn().expect("Something went wrong");

    // TODO stop services at will?
    spawn_handler(server_arc.clone(), handle, service_name.clone(), OutputKind::Compile, move |(server, success)| {
        let mut server = server.lock().unwrap();
        // Mark the service as no longer running when it exits
        server.system_state.service_statuses.get_mut(&service_name).unwrap().is_running = false;
        let broadcast = Broadcast::State(server.system_state.clone());
        server.broadcast_all(broadcast);
    });

    Some(())
}
