use crate::config::ExecutableEntry;
use crate::models::{OutputKey, OutputKind, OutputLine};
use log::{error, info};
use std::io::{BufRead, BufReader};
use std::ops::Neg;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};
use std::{io, thread};
use std::collections::VecDeque;
use std::thread::JoinHandle;
use crate::system_state::SystemState;

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
        // Substitute environment variables if placeholders are used in the env entry
        // TODO clean error handling, bubble error up and process in a nice way above
        let parsed = subst::substitute(value, &subst::Env)
            .expect(&format!("No variable found to substitute in env variable {}", value));

        cmd.env(key.clone(), parsed);
    });
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    // Set process group
    if cfg!(target_os = "linux") {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }

    cmd
}

pub struct ProcessHandler {
    pub handle: Arc<Mutex<Child>>,
    pub service_id: String,
    pub block_id: String,
    force_exit: Arc<Mutex<bool>>,
    pub exit_successful: Arc<Mutex<Option<bool>>>,
    pub buffered_outputs: Arc<Mutex<VecDeque<(OutputKey, String)>>>,
}
impl ProcessHandler {
    pub fn handle(process: Child, service_id: String, block_id: String) -> (ProcessHandler, Vec<(String, JoinHandle<()>)>) {
        let thread_prefix = format!("{service_id}/{block_id}");
        let handler = ProcessHandler {
            handle: Arc::new(Mutex::new(process)),
            service_id,
            block_id,
            force_exit: Arc::new(Mutex::new(false)),
            exit_successful: Arc::new(Mutex::new(None)),
            buffered_outputs: Arc::new(Mutex::new(VecDeque::new())),
        };

        let mut new_threads = vec![
            // Kill the process when the server exits and invoke the callback after the process finishes
            (
                format!("{thread_prefix}-manager"),
                {
                    let process_handle = handler.handle.clone();
                    let force_exit = handler.force_exit.clone();
                    let exit_status_arc = handler.exit_successful.clone();

                    thread::spawn(move || {
                        // Wait as long as the system and process are both running, or until an early-exit condition
                        // is fulfilled.
                        let mut killed = false;
                        loop {
                            if *force_exit.lock().unwrap() {
                                killed = true;
                                break;
                            }
                            if process_handle.lock().unwrap().try_wait().unwrap_or(None).is_some() {
                                break;
                            }
                            thread::sleep(Duration::from_millis(10));
                        }

                        let status = Self::kill_process(process_handle);
                        let success = status.as_ref().map_or(false, |status| status.success());

                        let mut exit_status = exit_status_arc.lock().unwrap();
                        *exit_status = Some(success);
                    })
                },
            ),
            // Read stdout
            (
                format!("{thread_prefix}-stdout"),
                {
                    let process_handle = handler.handle.clone();
                    let buffered_outputs = handler.buffered_outputs.clone();
                    let service_id = handler.service_id.clone();

                    thread::spawn(move || {
                        let stream = {
                            let mut handle = process_handle.lock().unwrap();
                            handle.stdout.take().unwrap()
                        };
                        let key = OutputKey::new(OutputKey::STD.into(), service_id.clone(), OutputKind::Run);

                        for line in BufReader::new(stream).lines() {
                            if let Ok(line) = line {
                                buffered_outputs.lock().unwrap().push_back((key.clone(), line));
                            }
                        }
                    })
                },
            ),
            // Read stderr
            (
                format!("{thread_prefix}-stderr"),
                {
                    let process_handle = handler.handle.clone();
                    let buffered_outputs = handler.buffered_outputs.clone();
                    let service_id = handler.service_id.clone();

                    thread::spawn(move || {
                        let stream = {
                            let mut handle = process_handle.lock().unwrap();
                            handle.stderr.take().unwrap()
                        };
                        let key = OutputKey::new(OutputKey::STD.into(), service_id.clone(), OutputKind::Run);

                        for line in BufReader::new(stream).lines() {
                            if let Ok(line) = line {
                                buffered_outputs.lock().unwrap().push_back((key.clone(), line));
                            }
                        }
                    })
                },
            )
        ];

        (handler, new_threads)
    }

    #[cfg(target_os = "linux")]
    fn kill_process(handle: Arc<Mutex<Child>>) -> io::Result<ExitStatus> {
        use nix::unistd::Pid;
        use nix::sys::signal::{self, Signal};

        let mut handle = handle.lock().unwrap();

        fn signal_and_wait(handle: &mut MutexGuard<Child>, signal: Signal, timeout: Duration) {
            info!("Sending {signal} to process group {pid}", pid = handle.id());
            if let Err(err) = signal::kill(Pid::from_raw((handle.id() as i32).neg()), signal) {
                error!("Failed to send {signal} to process: {err:?}")
            } else {
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
            }
        }


        // If the process is running, start by sending SIGINT
        if handle.try_wait().unwrap_or(None).is_none() {
            signal_and_wait(&mut handle, Signal::SIGINT, Duration::from_millis(5000))
        }

        // If the process is still running, then we should send a SIGTERM
        if handle.try_wait().unwrap_or(None).is_none() {
            signal_and_wait(&mut handle, Signal::SIGTERM, Duration::from_millis(5000))
        }

        // If the process is STILL running, then we should send a SIGKILL
        if handle.try_wait().unwrap_or(None).is_none() {
            signal_and_wait(&mut handle, Signal::SIGKILL, Duration::from_millis(5000))
        }

        // The process really should not be running anymore. But as a fallback, we use the kill()
        // function for handles
        if handle.try_wait().unwrap_or(None).is_none() {
            info!("Terminating process {pid} forcefully", pid = handle.id());
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

pub struct OnFinishParams {
    pub state: Arc<Mutex<SystemState>>,
    pub success: bool,
    pub exit_code: i32,
    pub killed: bool,
}

// TODO move?
pub trait CtrlOutputWriter {
    fn add_ctrl_output(&mut self, service_name: &str, str: String);
}
impl CtrlOutputWriter for MutexGuard<'_, SystemState> {
    fn add_ctrl_output(&mut self, service_name: &str, str: String) {
        self.add_output(
            &OutputKey {
                name: OutputKey::CTL.into(),
                service_ref: service_name.to_string(),
                kind: OutputKind::Run,
            },
            str,
        );
    }
}
