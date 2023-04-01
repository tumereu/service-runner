use std::{
    error::Error,
    fs::File,
    io::{BufReader, Result as IOResult, stdout},
    path::Path,
    thread,
    time::Duration
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
    widgets::{Block, Borders, Widget}
};
use tui::style::{Color, Style};

use shared::config::{read_config, Config};

use crate::app_state::{AppState, Phase};
use crate::input::process_inputs;
use crate::ui::render;

mod app_state;
mod ui;
mod input;

fn main() -> Result<(), Box<dyn Error>> {
    let config = read_config(Path::new("./config.toml"))?;
    let mut app_state = AppState {
        config,
        phase: Phase::Initializing
    };

    enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture
    )?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    while app_state.phase != Phase::Exit {
        render(&mut terminal, &app_state);
        process_inputs(&mut app_state);
        thread::sleep(Duration::from_millis(10));
    }

    // Clear terminal and restore normal mode
    terminal.clear()?;
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}