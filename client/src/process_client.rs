use std::future::Future;
use std::task::Poll;
use std::{env, thread};
use std::error::Error;
use shared::config::Config;
use crate::{ClientState, Status};
use reqwest::Client;
use std::process::{Command, Stdio};

pub async fn process_state(client_state: &mut ClientState, config: &Config, http: &Client) -> Result<(), Box<dyn Error>> {
    let port = config.server.port;

    match client_state.status {
        Status::CheckingServerStatus => {
            let response = http.get(format!("http://127.0.0.1:{port}/status")).send().await;

            match response {
                Ok(_) => {
                    client_state.status = Status::Idle
                }
                Err(_) => {
                    client_state.status = Status::StartingServer
                }
            }
        }
        Status::StartingServer => {
            Command::new(config.server.executable.clone())
                .current_dir(env::current_dir()?)
                .stdout(Stdio::null())
                .stdin(Stdio::null())
                .spawn()?;

            client_state.status = Status::CheckingServerStatus
        }
        Status::Finishing => {
            if config.server.daemon == false {
                http.post(format!("http://127.0.0.1:{port}/shutdown")).send().await;
            }

            client_state.status = Status::ReadyToExit;
        }
        _ => {

        }
    }

    Ok(())
}

fn start_server() {

}