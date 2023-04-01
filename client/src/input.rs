use std::fs::read;
use std::time::Duration;
use std::io::Result as IOResult;
use crossterm::event::{Event, KeyCode, poll as poll_events, read as read_event};
use crate::{AppState, Phase};

pub fn process_inputs(app_state: &mut AppState) -> IOResult<()> {
    let has_event = poll_events(Duration::from_millis(0))?;

    if has_event {
        if let Event::Key(key) = read_event()? {
            match key.code {
                KeyCode::Esc => {
                    app_state.phase = Phase::Exit;
                },
                _ => {}
            }
        }
    }

    Ok(())
}