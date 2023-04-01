use std::future::Future;
use std::task::Poll;
use std::thread;
use shared::config::Config;
use crate::{ClientState, Status};
use reqwest::Client;

pub async fn process_state(client_state: &mut ClientState, config: &Config, http: &Client) {
    let port = config.server.port;

    match client_state.status {
        Status::CheckingServerStatus => {
            let response = http.get(format!("http://127.0.0.1:{port}/status")).send().await;

            match response {
                Ok(_) => {
                    client_state.status = Status::Idle
                }
                _ => {
                    client_state.status = Status::StartingServer
                }
            }
        }
        Status::StartingServer => {
            
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
}

fn start_server() {

}