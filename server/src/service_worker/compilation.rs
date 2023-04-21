use shared::message::Broadcast;
use crate::service_worker::utils::{create_cmd, ProcessHandler};
use crate::ServerState;
use shared::message::models::{CompileStatus, ServiceAction, OutputKind, OutputKey};
use shared::system_state::Status;
use std::sync::{Mutex, Arc};
use shared::format_err;

pub fn handle_compilation(server_arc: Arc<Mutex<ServerState>>) -> Option<()> {
    let mut server = server_arc.lock().unwrap();

    // Do not spawn new compilations if any are any currently active.
    if server.get_state().service_statuses.values().any(|status| matches!(status.compile_status, CompileStatus::Compiling(_))) {
        return None
    }

    let (service_name, mut command, exec_display, index) = {
        let profile = server.get_state().current_profile.as_ref()?;
        let compilable = profile.services.iter()
            .filter(|service| service.compile.len() > 0)
            .find(|service| {
                let status = server.get_state().service_statuses.get(&service.name).unwrap();
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

        let status = server.get_state().service_statuses.get(&compilable.name).unwrap();
        let index = match status.compile_status {
            CompileStatus::None | CompileStatus::Failed => 0,
            CompileStatus::Compiled(index) => index + 1,
            CompileStatus::Compiling(_) => panic!("Should not exec this code with a compiling-status")
        };
        let exec_entry = compilable.compile.get(index).unwrap();
        let mut command = create_cmd(exec_entry, compilable.dir.as_ref());

        (compilable.name.clone(), command, format!("{exec_entry}"), index)
    };

    server.add_output(&OutputKey {
        name: OutputKey::CTRL.into(),
        service_ref: service_name.clone(),
        kind: OutputKind::Compile,
    }, format!("Exec: {exec_display}"));

    match command.spawn() {
        Ok(handle) => {
            server.update_state(|state| {
                state.service_statuses.get_mut(&service_name).unwrap().compile_status = CompileStatus::Compiling(index);
            });

            ProcessHandler {
                server: server_arc.clone(),
                handle,
                service_name: service_name.clone(),
                output: OutputKind::Compile,
                exit_early: |_| false,
                on_finish: move |(server, service_name, success)| {
                    let mut server = server.lock().unwrap();
                    if success {
                        server.update_service_status(&service_name, |status| {
                            status.compile_status = CompileStatus::Compiled(index);
                            status.action = ServiceAction::Restart;
                        });
                    } else {
                        server.update_service_status(&service_name, |status| {
                            status.compile_status = CompileStatus::Failed;
                            status.action = ServiceAction::None;
                        });

                        server.add_output(&OutputKey {
                            name: OutputKey::CTRL.into(),
                            service_ref: service_name.into(),
                            kind: OutputKind::Compile,
                        }, format!("Process exited with a non-zero status code"));
                    }
                }
            }.launch(&mut server);
        },
        Err(error) => {
            server.update_service_status(&service_name, |status| {
                status.compile_status = CompileStatus::Failed;
                status.action = ServiceAction::None;
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
