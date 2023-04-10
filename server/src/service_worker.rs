use std::arch::x86_64::_mm256_rcp_ps;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use shared::message::models::{ExecutableEntry, Service};
use shared::system_state::Status;
use crate::server_state::{Process, ServerState};

pub fn start_service_worker(state: Arc<Mutex<ServerState>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while state.lock().unwrap().system_state.status != Status::Exiting {
            work_services(state.clone());
            thread::sleep(Duration::from_millis(1))
        }
    })
}

fn work_services(state: Arc<Mutex<ServerState>>) -> Option<()> {
    let mut state = state.lock().unwrap();

    // Process already finished compilations
    loop {
        let mut finished: Option<usize> = None;
        for (index, process) in &mut state.compilations.iter_mut().enumerate() {
            if let Ok(Some(status)) = process.handle.try_wait() {
                finished = Some(index);
            }
        }

        if let Some(index) = finished {
            // TODO spawn next?
            let process = state.compilations.remove(index);
            println!("Process {process:?} exited");
        } else {
            break;
        }
        break;
    }

    // Do not spawn new compilations if one is currently active
    if !state.compilations.is_empty() {
        return None
    }
    
    let process = {
        let profile = state.system_state.current_profile.as_ref()?;
        let compilable = profile.services.iter()
            .find(|service| {
                let status = state.system_state.service_statuses.get(service.name()).unwrap();
                status.needs_compiling
            })?;

        match compilable {
            Service::Compilable { name, dir, compile, .. } => {
                let mut command = create_cmd(compile.first().unwrap(), dir);
                // TODO capture output
                command.stdout(Stdio::null());
                command.stderr(Stdio::null());

                // TODO handle erroneous commands?
                let handle = command.spawn().expect("Something went wrong");

                Process {
                    handle,
                    index: 0,
                    service: name.clone()
                }
            }
        }
    };

    state.system_state.service_statuses.get_mut(&process.service).unwrap().is_compiling = true;
    state.system_state.service_statuses.get_mut(&process.service).unwrap().needs_compiling = false;
    state.compilations.push(process);

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