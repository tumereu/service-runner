use std::io::{BufRead, BufReader};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex, MutexGuard};
use std::{io, thread};
use std::time::{Duration, Instant};
use nix::libc::stat;
use shared::dbg_println;

use shared::message::models::{ExecutableEntry, OutputKey, OutputKind};
use shared::message::Broadcast;
use shared::system_state::Status;

use crate::server_state::ServerState;

pub fn create_cmd<S>(entry: &ExecutableEntry, dir: Option<S>) -> Command
where
    S: AsRef<str>,
{
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

pub struct ProcessHandler<F, G>
where
    F: FnOnce(OnFinishParams) + Send + 'static,
    G: Fn((Arc<Mutex<ServerState>>, &str)) -> bool + Send + 'static,
{
    pub server: Arc<Mutex<ServerState>>,
    pub handle: Arc<Mutex<Child>>,
    pub service_name: String,
    pub output: OutputKind,
    pub on_finish: F,
    pub exit_early: G,
}

impl<F, G> ProcessHandler<F, G>
where
    F: FnOnce(OnFinishParams) + Send + 'static,
    G: Fn((Arc<Mutex<ServerState>>, &str)) -> bool + Send + 'static,
{
    pub fn launch(self) {
        let ProcessHandler {
            server,
            handle,
            service_name,
            output,
            on_finish,
            exit_early,
        } = self;
        let mut new_threads = vec![
            // Kill the process when the server exits and invoke the callback after the process finishes
            {
                let handle = handle.clone();
                let server = server.clone();
                let service_name = service_name.clone();
                thread::spawn(move || {
                    // Wait as long as the server and the process are both running, or until an early-exit condition is
                    // fulfilled.
                    let mut killed = false;
                    loop {
                        if exit_early((server.clone(), &service_name)) {
                            killed = true;
                            break;
                        }
                        if handle.lock().unwrap().try_wait().unwrap_or(None).is_some() {
                            break;
                        }
                        if server.lock().unwrap().get_state().status == Status::Exiting {
                            break;
                        }
                        thread::sleep(Duration::from_millis(10));
                    }
                    let status = Self::kill_process(handle);
                    let success = status.as_ref().map_or(false, |status| status.success());
                    on_finish(
                        OnFinishParams {
                            server,
                            service_name: &service_name,
                            success,
                            exit_code: status.map(|status| status.code().unwrap_or(0)).unwrap_or(0),
                            killed
                        }
                    )
                })
            },
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
                    let key = OutputKey::new(OutputKey::STD.into(), service_name, output.clone());

                    for line in BufReader::new(stream).lines() {
                        if let Ok(line) = line {
                            Self::process_output_line(server.clone(), &key, line);
                        }
                    }
                })
            },
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
                    let key = OutputKey::new(OutputKey::STD.into(), service_name, output.clone());

                    for line in BufReader::new(stream).lines() {
                        if let Ok(line) = line {
                            Self::process_output_line(server.clone(), &key, line);
                        }
                    }
                })
            },
        ];

        server
            .lock()
            .unwrap()
            .active_threads
            .append(&mut new_threads);
    }

    fn process_output_line(state: Arc<Mutex<ServerState>>, key: &OutputKey, output: String) {
        let mut server = state.lock().unwrap();

        // Store the line locally so that it can be sent to clients that connect later
        let line = server.output_store.add_output(&key, output).clone();

        // But also broadcast the line to all clients
        server.broadcast_all(Broadcast::OutputLine(key.clone(), line));
    }

    #[cfg(target_os = "linux")]
    fn kill_process(handle: Arc<Mutex<Child>>) -> io::Result<ExitStatus> {
        use nix::unistd::Pid;
        use nix::sys::signal::{self, Signal};

        let mut handle = handle.lock().unwrap();

        fn signal_and_wait(handle: &mut MutexGuard<Child>, signal: Signal, timeout: Duration) -> io::Result<()> {
            signal::kill(Pid::from_raw(handle.id() as i32), signal)?;
            let signal_sent = Instant::now();

            // Wait for the process to finish, up to a limit
            loop {
                if Instant::now().duration_since(signal_sent) > timeout {
                    break;
                }
                if handle.try_wait().unwrap_or(None).is_some() {
                    break;
                }
                thread::sleep(Duration::from_millis(10));
            }

            Ok(())
        }


        // If the process is running, start by sending SIGINT
        if handle.try_wait().unwrap_or(None).is_none() {
            if let Err(err) = signal_and_wait(&mut handle, Signal::SIGINT, Duration::from_millis(5000)) {
                dbg_println!("Failed to send SIGINT to process: {err:?}")
            }
        }

        // If the process is still running, then we should send a SIGTERM
        if handle.try_wait().unwrap_or(None).is_none() {
            if let Err(err) = signal_and_wait(&mut handle, Signal::SIGTERM, Duration::from_millis(5000)) {
                dbg_println!("Failed to send SIGTERM to process: {err:?}")
            }
        }

        // Finally, if the process is still alive, we should kill it forcefully
        if handle.try_wait().unwrap_or(None).is_none() {
            // TODO kill the process group? Must be set when starting the process though
            handle.kill().unwrap_or(());
        }
        // Obtain exit status and invoke callback
        handle.wait()
    }

    #[cfg(not(target_os = "linux"))]
    fn kill_process(handle: Arc<Mutex<Child>>) -> io::Result<ExitStatus> {
        let mut handle = handle.lock().unwrap();
        // Kill the process if it its alive
        // TODO graceful terminate? Kill children somehow
        handle.kill().unwrap_or(());
        // Obtain exit status and invoke callback
        handle.wait()
    }
}

pub struct OnFinishParams<'a> {
    pub server: Arc<Mutex<ServerState>>,
    pub service_name: &'a str,
    pub success: bool,
    pub exit_code: i32,
    pub killed: bool,
}