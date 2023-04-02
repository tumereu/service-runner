use std::error::Error;
use std::fs::read;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use crossterm::event::{Event, KeyCode, poll as poll_events, read as read_event};
use reqwest::Client;
use shared::config::Config;
use crate::{ClientState, Status};

pub async fn process_inputs(state: Arc<Mutex<ClientState>>, config: &Config, http: &Client) -> Result<(), String> {
    if poll_events(Duration::from_millis(0)).unwrap() {
        let port = config.server.port;
        let event = read_event().unwrap();

        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Esc => {
                    if config.server.daemon == false {
                        http.post(format!("http://127.0.0.1:{port}/shutdown")).send().await;
                    }

                    let mut state = state.lock().unwrap();
                    state.status = Status::Exiting;
                },
                _ => {}
            }
        }
    }

    Ok(())
}