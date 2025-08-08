use std::io::{BufRead, BufReader};
use std::ops::Neg;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};
use std::{io, thread};

use log::{error, info};
use nix::libc::SIGKILL;

use crate::config::ExecutableEntry;
use crate::models::{OutputKey, OutputKind};
use crate::system_state::SystemState;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ConcurrentOperationStatus {
    Running,
    Ok,
    Failed,
}

pub enum ConcurrentOperationHandle {
    Process(ProcessWrapper),
    Work(WorkWrapper),
}
impl ConcurrentOperationHandle {
    pub fn status(&self) -> ConcurrentOperationStatus {
        match self {
            ConcurrentOperationHandle::Process(wrapper) => wrapper.status.lock().unwrap().clone(),
            ConcurrentOperationHandle::Work(wrapper) => wrapper.status.lock().unwrap().clone(),
        }
    }

    /// Signals to this operation that it should stop
    pub fn stop(&self) {
        match self {
            ConcurrentOperationHandle::Process(wrapper) => wrapper.stop(),
            ConcurrentOperationHandle::Work(_) => {
                // Work is intended for short-lived operations, and cannot be stopped. It should
                // finish soon on its own
            }
        }
    }
}

pub struct WorkWrapper {
    pub status: Arc<Mutex<ConcurrentOperationStatus>>,
}
impl WorkWrapper {
    pub fn wrap<F>(
        state_arc: Arc<Mutex<SystemState>>,
        service_id: Option<String>,
        work_name: String,
        silent: bool,
        work: F
    ) -> WorkWrapper where F : (FnOnce() -> WorkResult) + Send + 'static {
        let wrapper = WorkWrapper {
            status: Arc::new(Mutex::new(ConcurrentOperationStatus::Running)),
        };
        let status = wrapper.status.clone();
        let state_arc_copy = state_arc.clone();
        let output_key = OutputKey {
            service_id: service_id.clone(),
            source_name: work_name.clone(),
            kind: OutputKind::System,
        };
        let full_name = service_id.map(|id| format!("{id}.{work_name}")).unwrap_or(work_name);

        let thread = thread::spawn(move || {
            let result = work();

            if result.successful {
                *status.lock().unwrap() = ConcurrentOperationStatus::Ok
            } else {
                *status.lock().unwrap() = ConcurrentOperationStatus::Failed
            }

            if !silent {
                let mut state = state_arc_copy.lock().unwrap();
                result.output.into_iter().for_each(|output| {
                    state.add_output(&output_key, output);
                });
            }
        });

        {
            let mut state = state_arc.lock().unwrap();
            state.active_threads.push((
                format!("{full_name}-work"),
                thread
            ));
        }
        
        wrapper
    }
}

pub struct WorkResult {
    pub successful: bool,
    pub output: Vec<String>
}

pub struct ProcessWrapper {
    pub handle: Arc<Mutex<Child>>,
    pub service_id: Option<String>,
    pub work_name: String,
    pub status: Arc<Mutex<ConcurrentOperationStatus>>,
    force_exit: Arc<Mutex<bool>>,
}
impl ProcessWrapper {
    pub fn wrap(
        state_arc: Arc<Mutex<SystemState>>,
        service_id: Option<String>,
        work_name: String,
        process: Child,
    ) -> ProcessWrapper {
        let handler = ProcessWrapper {
            handle: Arc::new(Mutex::new(process)),
            service_id: service_id.clone(),
            work_name: work_name.clone(),
            force_exit: Arc::new(Mutex::new(false)),
            status: Arc::new(Mutex::new(ConcurrentOperationStatus::Running)),
        };
        let full_name = service_id.map(|id| format!("{id}.{work_name}")).unwrap_or(work_name.clone());

        let mut new_threads = vec![
            // Kill the process when the server exits and invoke the callback after the process finishes
            (
                format!("{full_name}-manager"),
                {
                    let process_handle = handler.handle.clone();
                    let force_exit = handler.force_exit.clone();
                    let status_arc = handler.status.clone();
                    let state_arc = state_arc.clone();

                    thread::spawn(move || {
                        // Wait as long as the system and process are both running, or until an early-exit condition
                        // is fulfilled.
                        let mut killed = false;
                        loop {
                            if *force_exit.lock().unwrap() {
                                killed = true;
                                break;
                            }
                            if state_arc.lock().unwrap().should_exit {
                                killed = true;
                                break;
                            }
                            if process_handle.lock().unwrap().try_wait().unwrap_or(None).is_some() {
                                break;
                            }
                            thread::sleep(Duration::from_millis(10));
                        }

                        let system_exiting = state_arc.lock().unwrap().should_exit;
                        let status = Self::kill_process(process_handle, !system_exiting);
                        let success = status.as_ref().map_or(false, |status| status.success());

                        let mut exit_status = status_arc.lock().unwrap();
                        *exit_status = if success {
                            ConcurrentOperationStatus::Ok
                        } else {
                            ConcurrentOperationStatus::Failed
                        }
                    })
                },
            ),
            // Read stdout
            (
                format!("{full_name}-stdout"),
                {
                    let process_handle = handler.handle.clone();
                    let state_arc = state_arc.clone();
                    let service_id = handler.service_id.clone();
                    let output_key = OutputKey {
                        service_id: service_id.clone(),
                        source_name: work_name.clone(),
                        kind: OutputKind::ExtProcess,
                    };

                    thread::spawn(move || {
                        let stream = {
                            let mut handle = process_handle.lock().unwrap();
                            handle.stdout.take().unwrap()
                        };

                        for line in BufReader::new(stream).lines() {
                            if let Ok(line) = line {
                                let mut state = state_arc.lock().unwrap();
                                state.output_store.add_output(&output_key, line);
                            }
                        }
                    })
                },
            ),
            // Read stderr
            (
                format!("{full_name}-stderr"),
                {
                    let process_handle = handler.handle.clone();
                    let state_arc = state_arc.clone();
                    let service_id = handler.service_id.clone();
                    let output_key = OutputKey {
                        service_id: service_id.clone(),
                        source_name: work_name.clone(),
                        kind: OutputKind::ExtProcess,
                    };

                    thread::spawn(move || {
                        let stream = {
                            let mut handle = process_handle.lock().unwrap();
                            handle.stderr.take().unwrap()
                        };

                        for line in BufReader::new(stream).lines() {
                            if let Ok(line) = line {
                                let mut state = state_arc.lock().unwrap();
                                state.output_store.add_output(&output_key, line);
                            }
                        }
                    })
                },
            )
        ];

        {
            let mut state = state_arc.lock().unwrap();
            state.active_threads.append(&mut new_threads);
        }

        handler
    }

    pub fn stop(&self) {
        *self.force_exit.lock().unwrap() = true;
    }

    #[cfg(target_os = "linux")]
    fn kill_process(handle: Arc<Mutex<Child>>, be_nice: bool) -> io::Result<ExitStatus> {
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

        if be_nice {
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
        } else {
            info!("Sending {signal} to process group {pid}", signal = Signal::SIGKILL, pid = handle.id());
            match signal::kill(Pid::from_raw((handle.id() as i32).neg()), Signal::SIGKILL) {
                Err(error) => error!("Failed to send signal to process: {error:?}"),
                Ok(_) => {},
            }
        }

        // The process really should not be running anymore. But as a fallback, use the handle's kill() function.
        if handle.try_wait().unwrap_or(None).is_none() {
            info!("Terminating process {pid} forcefully", pid = handle.id());
            handle.kill().unwrap_or(());
        }
        // Obtain exit status and invoke callback
        handle.wait()
    }

    #[cfg(not(target_os = "linux"))]
    fn kill_process(handle: Arc<Mutex<Child>>, be_nice: bool) -> io::Result<ExitStatus> {
        let mut handle = handle.lock().unwrap();
        // Kill the process if it its alive
        // TODO graceful terminate? Kill children somehow
        handle.kill().unwrap_or(());
        // Obtain exit status and invoke callback
        handle.wait()
    }
}
