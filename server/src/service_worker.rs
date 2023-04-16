use std::io::{BufReader, BufRead};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration};
use shared::message::Broadcast;
use shared::message::models::{CompileStatus, ExecutableEntry, OutputKey, OutputKind, Service};
use shared::system_state::{Status};
use crate::server_state::{ServerState};

pub fn start_service_worker(state: Arc<Mutex<ServerState>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while state.lock().unwrap().system_state.status != Status::Exiting {
            work_services(state.clone());
            thread::sleep(Duration::from_millis(1))
        }
    })
}

fn work_services(server_arc: Arc<Mutex<ServerState>>) -> Option<()> {
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

        command.stdin(Stdio::null());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

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

fn create_cmd<S>(
    entry: &ExecutableEntry,
    dir: Option<S>
) -> Command where S: AsRef<str> {
    let mut cmd = Command::new(entry.executable.clone());
    cmd.args(entry.args.clone());
    if let Some(dir) = dir {
        cmd.current_dir(dir.as_ref());
    }
    entry.env.iter().for_each(|(key, value)| {
        cmd.env(key.clone(), value.clone());
    });

    cmd
}

fn spawn_handler<F>(
    server: Arc<Mutex<ServerState>>,
    handle: Child,
    service_name: String,
    process_kind: OutputKind,
    on_finish: F
) where F: FnOnce((Arc<Mutex<ServerState>>, bool)) + Send + 'static {
    let handle = Arc::new(Mutex::new(handle));

    // Kill the process when the server exits and invoke the callback after the process finishes
    {
        let handle = handle.clone();
        let server = server.clone();
        thread::spawn(move || {
            // Wait as long as the server and the process are both running
            while server.lock().unwrap().system_state.status != Status::Exiting
                && handle.lock().unwrap().try_wait().unwrap_or(None).is_none() {
                thread::sleep(Duration::from_millis(1));
            }

            let mut handle = handle.lock().unwrap();
            // Kill the process if it its alive
            handle.kill().unwrap_or(());
            // Obtain exit status and invoke callback
            let success = handle.wait().map_or(false, |status| status.success());
            on_finish((server, success));
        });
    }

    // Read stdout
    {
        let handle = handle.clone();
        let server = server.clone();
        let service_name = service_name.clone();
        thread::spawn(move || {
            let stream = {
                let mut handle = handle.lock().unwrap();
                handle.stdout.take().unwrap()
            };
            let key = OutputKey::new("std".into(), service_name, process_kind.clone());

            for line in BufReader::new(stream).lines() {
                if let Ok(line) = line {
                    process_output_line(server.clone(), &key, line);
                }
            }
        });
    }

    // Read stderr
    {
        let handle = handle.clone();
        let server = server.clone();
        let service_name = service_name.clone();
        thread::spawn(move || {
            let stream = {
                let mut handle = handle.lock().unwrap();
                handle.stderr.take().unwrap()
            };
            let key = OutputKey::new("std".into(), service_name, process_kind.clone());

            for line in BufReader::new(stream).lines() {
                if let Ok(line) = line {
                    process_output_line(server.clone(), &key, line);
                }
            }
        });
    }
}

fn process_output_line(
    state: Arc<Mutex<ServerState>>,
    key: &OutputKey,
    output: String
) {
    let mut server = state.lock().unwrap();

    // Store the line locally so that it can be sent to clients that connect later
    let line = server.output_store.add_output(&key, output).clone();

    // But also broadcast the line to all clients
    server.broadcast_all(Broadcast::OutputLine(key.clone(), line));
}
