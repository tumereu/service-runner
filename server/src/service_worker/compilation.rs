use shared::message::Broadcast;
use crate::service_worker::utils::{create_cmd, ProcessHandler};
use crate::ServerState;
use shared::message::models::{CompileStatus, ServiceAction, OutputKind, OutputKey};
use shared::system_state::Status;
use std::sync::{Mutex, Arc};

pub fn handle_compilation(server_arc: Arc<Mutex<ServerState>>) -> Option<()> {
    let mut server = server_arc.lock().unwrap();

    // Do not spawn new compilations if any are currently active.
    // TODO support parallel compilation?
    if server.active_compile_count > 0 {
        return None
    }

    let (service_name, mut command, index) = {
        let profile = server.system_state.current_profile.as_ref()?;
        let compilable = profile.services.iter()
            .filter(|service| service.compile.len() > 0)
            .find(|service| {
                let status = server.system_state.service_statuses.get(&service.name).unwrap();
                match status.compile_status {
                    // Services with no compile steps executed should be compiled
                    CompileStatus::None | CompileStatus::Failed => match status.action {
                        ServiceAction::Recompile =>  true,
                        _ => false
                    },
                    // Services with some but not all compile-steps should be compiled
                    CompileStatus::Compiled(index) => index < service.compile.len() - 1,
                    // Services currently compiling should not be compiled
                    CompileStatus::Compiling(_) => false
                }
            })?;

        let status = server.system_state.service_statuses.get(&compilable.name).unwrap();
        let index = match status.compile_status {
            CompileStatus::None | CompileStatus::Failed => 0,
            CompileStatus::Compiled(index) => index + 1,
            CompileStatus::Compiling(_) => panic!("Should not exec this code with a compiling-status")
        };
        let mut command = create_cmd(compilable.compile.get(index).unwrap(), compilable.dir.as_ref());

        (compilable.name.clone(), command, index)
    };

    match command.spawn() {
        Ok(handle) => {
            server.active_compile_count += 1;
            let mut status = server.system_state.service_statuses.get_mut(&service_name).unwrap();
            status.compile_status = CompileStatus::Compiling(index);
            let broadcast = Broadcast::State(server.system_state.clone());
            server.broadcast_all(broadcast);

            ProcessHandler {
                server: server_arc.clone(),
                handle,
                service_name: service_name.clone(),
                output: OutputKind::Compile,
                exit_early: |_| false,
                on_finish: move |(server, service_name, success)| {
                    let mut server = server.lock().unwrap();
                    server.active_compile_count -= 1;
                    let mut status = server.system_state.service_statuses.get_mut(service_name).unwrap();
                    if success {
                        status.compile_status = CompileStatus::Compiled(index);
                        status.action = ServiceAction::Restart;
                    } else {
                        status.compile_status = CompileStatus::Failed;
                        status.action = ServiceAction::None;
                    }
                    let broadcast = Broadcast::State(server.system_state.clone());
                    server.broadcast_all(broadcast);
                }
            }.launch(&mut server);
        },
        Err(error) => {
            let mut status = server.system_state.service_statuses.get_mut(&service_name).unwrap();
            status.compile_status = CompileStatus::Failed;
            status.action = ServiceAction::None;
            server.add_output(&OutputKey {
                name: OutputKey::CTRL.into(),
                service_ref: service_name,
                kind: OutputKind::Compile,
            }, format!("{error}"));
            let broadcast = Broadcast::State(server.system_state.clone());
            server.broadcast_all(broadcast);
        }
    }

    Some(())
}
