use shared::message::Broadcast;
use crate::service_worker::utils::{spawn_handler, create_cmd};
use crate::ServerState;
use shared::message::models::{CompileStatus, OutputKind};
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
                    CompileStatus::None => true,
                    // Services with some but not all compile-steps should be compiled
                    CompileStatus::Compiled(index) => index < service.compile.len() - 1,
                    // Services currently compiling should not be compiled
                    CompileStatus::Compiling(_) => false
                }
            })?;

        let status = server.system_state.service_statuses.get(&compilable.name).unwrap();
        let index = match status.compile_status {
            CompileStatus::None => 0,
            CompileStatus::Compiled(index) => index + 1,
            CompileStatus::Compiling(_) => panic!("Should not exec this code with a compiling-status")
        };
        let mut command = create_cmd(compilable.compile.get(index).unwrap(), compilable.dir.as_ref());

        (compilable.name.clone(), command, index)
    };

    // TODO handle erroneous commands?
    server.active_compile_count += 1;
    server.system_state.service_statuses.get_mut(&service_name).unwrap().compile_status = CompileStatus::Compiling(index);
    let broadcast = Broadcast::State(server.system_state.clone());
    server.broadcast_all(broadcast);

    let handle = command.spawn().expect("Something went wrong");

    spawn_handler(server_arc.clone(), handle, service_name.clone(), OutputKind::Compile, move |(state, success)| {
        let mut state = state.lock().unwrap();
        state.active_compile_count -= 1;
        state.system_state.service_statuses.get_mut(&service_name).unwrap().compile_status = CompileStatus::Compiled(index);
        let broadcast = Broadcast::State(state.system_state.clone());
        state.broadcast_all(broadcast);
    });

    Some(())
}
