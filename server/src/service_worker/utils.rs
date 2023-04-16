use std::io::{BufReader, BufRead};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration};
use shared::message::Broadcast;
use shared::message::models::{CompileStatus, ExecutableEntry, OutputKey, OutputKind, Service};
use shared::system_state::{Status};
use crate::server_state::{ServerState};

pub fn create_cmd<S>(
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
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    cmd
}

pub fn spawn_handler<F>(
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

pub fn process_output_line(
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
