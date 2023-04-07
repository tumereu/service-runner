use std::env;
use std::error::Error;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use reqwest::Client;
use shared::config::Config;

use crate::{ClientState, Status};

pub fn connect_to_server(state: Arc<Mutex<ClientState>>, config: Arc<Config>) -> Result<(), String> {
    let port = config.server.port;
    let status = state.lock().unwrap().status;

    match status {
        Status::CheckingServerStatus => {
            // TODO open socket
        }
        Status::StartingServer => {
            Command::new(config.server.executable.clone())
                .arg(&config.conf_dir)
                .current_dir(env::current_dir().map_err(|err| {
                    let msg = err.to_string();
                    format!("Failed to read current workdir: {msg}")
                })?)
                .stdout(Stdio::null())
                .stdin(Stdio::null())
                .spawn()
                .map_err(|err| {
                    let msg = err.to_string();
                    format!("Failed to spawn server process: {msg}")
                })?;

            let mut state = state.lock().unwrap();
            state.status = Status::CheckingServerStatus
        }
        Status::Ready => {
        }
        _ => {

        }
    }

    Ok(())
}

fn start_server() {

}