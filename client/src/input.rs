use std::fs::read;
use std::time::Duration;
use std::io::Result as IOResult;
use std::sync::{Arc, Mutex};
use crossterm::event::{Event, KeyCode, poll as poll_events, read as read_event};
use crate::{ClientState, Status};

pub fn process_inputs(state: Arc<Mutex<ClientState>>) -> IOResult<()> {
    let has_event = poll_events(Duration::from_millis(0))?;

    if has_event {
        if let Event::Key(key) = read_event()? {
            match key.code {
                KeyCode::Esc => {
                    let mut state = state.lock().unwrap();
                    state.status = Status::Finishing;
                },
                _ => {}
            }
        }
    }

    Ok(())
}