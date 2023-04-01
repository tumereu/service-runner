use std::{env, error::Error, fs::File, io::{BufReader, Result as IOResult, stdout}, path::Path, thread, time::Duration};

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
use crate::process_client::process_state;

use reqwest::Client;

use crate::client_state::{ClientState, Status};
use crate::input::process_inputs;
use crate::ui::render;

mod client_state;
mod ui;
mod input;
mod process_client;

fn main() -> Result<(), Box<dyn Error>> {
    let config = read_config(Path::new("./config.toml"))?;

    let mut client_state = ClientState::new();
    let http_client = Client::builder()
        .timeout(Duration::from_millis(1000))
        .build()?;

    let async_runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture
    )?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        process_inputs(&mut client_state)?;
        async_runtime.block_on(async {
            process_state(&mut client_state, &config, &http_client).await
        })?;
        render(&mut terminal, &client_state)?;

        if client_state.status == Status::ReadyToExit {
            break;
        } else {
            thread::sleep(Duration::from_millis(10));
        }
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