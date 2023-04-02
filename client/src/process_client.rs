use std::{env};
use std::error::Error;
use shared::config::Config;
use crate::{ClientState, Status};
use reqwest::Client;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub async fn process_state(state: Arc<Mutex<ClientState>>, config: &Config, http: &Client) -> Result<(), String> {
    let port = config.server.port;
    let status = state.lock().unwrap().status;

    match status {
        Status::CheckingServerStatus => {
            let response = http.get(format!("http://127.0.0.1:{port}/status")).send().await;

            let mut state = state.lock().unwrap();
            match response {
                Ok(_) => {
                    state.status = Status::Ready
                }
                Err(_) => {
                    state.status = Status::StartingServer
                }
            }
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
            let last_status_check = state.lock().unwrap().last_status_check;

            if Instant::now().duration_since(last_status_check) >= Duration::from_millis(100) {
                let response = http.get(format!("http://127.0.0.1:{port}/status")).send().await;

                match response {
                    Ok(resp) => {
                        let text = resp.text().await.map_err(|err| {
                            let msg = err.to_string();
                            format!("Failed to read response: {msg}")
                        })?;

                        let mut state = state.lock().unwrap();
                        state.system = Some(
                            serde_json::from_str(&text)
                                .map_err(|err| {
                                    let msg = err.to_string();
                                    format!("Failed to parse response as JSON: {msg}")
                                })?
                        );
                        state.last_status_check = Instant::now();
                    }
                    Err(_) => {
                        let mut state = state.lock().unwrap();
                        state.status = Status::StartingServer
                    }
                }
            }
        }
        _ => {

        }
    }

    Ok(())
}

fn start_server() {

}