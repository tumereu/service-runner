use std::future::Future;
use std::task::Poll;
use std::{env, thread};
use std::error::Error;
use shared::config::Config;
use crate::{ClientState, Status};
use reqwest::Client;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

pub async fn process_state(state: Arc<Mutex<ClientState>>, config: &Config, http: &Client) -> Result<(), Box<dyn Error>> {
    let port = config.server.port;
    let status = state.lock().unwrap().status;

    match status {
        Status::CheckingServerStatus => {
            let response = http.get(format!("http://127.0.0.1:{port}/status")).send().await;

            let mut state = state.lock().unwrap();
            match response {
                Ok(_) => {
                    state.status = Status::Idle
                }
                Err(_) => {
                    state.status = Status::StartingServer
                }
            }
        }
        Status::StartingServer => {
            Command::new(config.server.executable.clone())
                .arg(&config.conf_dir)
                .current_dir(env::current_dir()?)
                .stdout(Stdio::null())
                .stdin(Stdio::null())
                .spawn()?;

            let mut state = state.lock().unwrap();
            state.status = Status::CheckingServerStatus
        }
        Status::Finishing => {
            if config.server.daemon == false {
                http.post(format!("http://127.0.0.1:{port}/shutdown")).send().await;
            }

            let mut state = state.lock().unwrap();
            state.status = Status::ReadyToExit;
        }
        _ => {

        }
    }

    Ok(())
}

fn start_server() {

}