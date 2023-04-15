use std::arch::x86_64::_mm256_rcp_ps;
use std::io::{BufReader, BufRead};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use shared::message::Broadcast;
use shared::message::models::{CompileStatus, ExecutableEntry, Service};
use shared::system_state::{Status, SystemState};
use crate::server_state::{ServerState};

pub fn start_service_worker(state: Arc<Mutex<ServerState>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while state.lock().unwrap().system_state.status != Status::Exiting {
            work_services(state.clone());
            thread::sleep(Duration::from_millis(1))
        }
    })
}

fn work_services(state_arc: Arc<Mutex<ServerState>>) -> Option<()> {
    let mut state = state_arc.lock().unwrap();

    // Do not spawn new compilations if any are currently active.
    // TODO support parallel compilation?
    if state.active_compile_count > 0 {
        return None
    }

    let (service_name, mut command, index) = {
        let profile = state.system_state.current_profile.as_ref()?;
        let compilable = profile.services.iter()
            .find(|service| {
                match service {
                    Service::Compilable { compile, .. } => {
                        let status = state.system_state.service_statuses.get(service.name()).unwrap();
                        match status.compile_status {
                            // Services with no compile steps executed should be compiled
                            CompileStatus::None => true,
                            // Services with some but not all compile-steps should be compiled
                            CompileStatus::Compiled(index) => index < compile.len() - 1,
                            // Services currently compiling should not be compiled
                            CompileStatus::Compiling(_) => false
                        }
                    }
                }
            })?;

        match compilable {
            Service::Compilable { name, dir, compile, .. } => {
                let status = state.system_state.service_statuses.get(name).unwrap();
                let index = match status.compile_status {
                    CompileStatus::None => 0,
                    CompileStatus::Compiled(index) => index + 1,
                    CompileStatus::Compiling(_) => panic!("Should not exec this code with a compiling-status")
                };
                let mut command = create_cmd(compile.get(index).unwrap(), dir);

                command.stdin(Stdio::null());
                command.stdout(Stdio::piped());
                command.stderr(Stdio::piped());

                (name.clone(), command, index)
            }
        }
    };

    // TODO handle erroneous commands?
    state.active_compile_count += 1;
    state.system_state.service_statuses.get_mut(&service_name).unwrap().compile_status = CompileStatus::Compiling(index);
    state.broadcast_all(Broadcast::State(state.system_state.clone()));

    let handle = command.spawn().expect("Something went wrong");

    spawn_handler(state_arc.clone(), handle, move |(state, success)| {
        let mut state = state.lock().unwrap();
        state.active_compile_count -= 1;
        state.system_state.service_statuses.get_mut(&service_name).unwrap().compile_status = CompileStatus::Compiled(index);
        state.broadcast_all(Broadcast::State(state.system_state.clone()));
    });

    Some(())
}

fn create_cmd(
    entry: &ExecutableEntry,
    dir: &str
) -> Command {
    let mut cmd = Command::new(entry.executable.clone());
    cmd.args(entry.args.clone());
    cmd.current_dir(dir);
    entry.env.iter().for_each(|(key, value)| {
        cmd.env(key.clone(), value.clone());
    });

    cmd
}

fn spawn_handler<F>(
    state: Arc<Mutex<ServerState>>,
    handle: Child,
    on_finish: F
) where F: FnOnce((Arc<Mutex<ServerState>>, bool)) + Send + 'static {
    let handle = Arc::new(Mutex::new(handle));

    // Kill the process when the server exits and invoke the callback after the process finishes
    {
        let handle = handle.clone();
        thread::spawn(move || {
            // Wait as long as the server and the process are both running
            while state.lock().unwrap().system_state.status != Status::Exiting
                && handle.lock().unwrap().try_wait().unwrap_or(None).is_none() {
                thread::sleep(Duration::from_millis(1));
            }

            let mut handle = handle.lock().unwrap();
            // Kill the process if it its alive
            handle.kill().unwrap_or(());
            // Obtain exit status and invoke callback
            let success = handle.wait().map_or(false, |status| status.success());
            on_finish((state, success));
        });
    }

    // Read stdout
    {
        let handle = handle.clone();
        thread::spawn(move || {
            let stream = {
                let mut handle = handle.lock().unwrap();
                handle.stdout.take().unwrap()
            };

            for line in BufReader::new(stream).lines() {
                println!("STDOUT: {line:?}")
            }
        });
    }

    // Read stderr
    {
        let handle = handle.clone();
        thread::spawn(move || {
            let stream = {
                let mut handle = handle.lock().unwrap();
                handle.stderr.take().unwrap()
            };

            for line in BufReader::new(stream).lines() {
                println!("STDERR: {line:?}")
            }
        });
    }
}