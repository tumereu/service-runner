use std::fs::read;
use std::time::Duration;
use std::io::Result as IOResult;
use crossterm::event::{Event, KeyCode, poll as poll_events, read as read_event};
use crate::{ClientState, Status};

pub fn process_inputs(client_state: &mut ClientState) -> IOResult<()> {
    let has_event = poll_events(Duration::from_millis(0))?;

    if has_event {
        if let Event::Key(key) = read_event()? {
            match key.code {
                KeyCode::Esc => {
                    client_state.status = Status::Finishing;
                },
                _ => {}
            }
        }
    }

    Ok(())
}