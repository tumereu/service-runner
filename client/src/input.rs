

use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossterm::event::{Event, KeyCode, poll as poll_events, read as read_event};

use shared::config::Config;

use crate::{ClientState, Status};

pub fn process_inputs(state: Arc<ClientState>) -> Result<(), String> {
    let config = state.config.clone();

    if poll_events(Duration::from_millis(0)).unwrap() {
        let _port = config.server.port;
        let event = read_event().unwrap();

        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Esc => {
                    // TODO send shutdown message

                    *state.status.lock().unwrap() = Status::Exiting;
                },
                _ => {}
            }
        }
    }

    Ok(())
}