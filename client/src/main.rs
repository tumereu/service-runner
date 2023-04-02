use std::{env, error::Error, fs::File, io::{BufReader, Result as IOResult, stdout}, path::Path, thread, time::Duration};
use std::sync::{Arc, Mutex};

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
use tokio::runtime::Runtime;
use tui::backend::Backend;

use crate::client_state::{ClientState, Status};
use crate::input::process_inputs;
use crate::ui::render;

mod client_state;
mod ui;
mod input;
mod process_client;

fn main() -> Result<(), Box<dyn Error>> {
    let config_dir: String = env::args().collect::<Vec<String>>()
        .get(1)
        .ok_or("Specify the configuration directory in order to run the app")?
        .clone();

    let config = read_config(&config_dir)?;

    let mut client_state = Arc::new(Mutex::new(ClientState::new()));
    let http_client = Client::builder()
        .timeout(Duration::from_millis(1000))
        .build()?;

    let async_runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
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
    let mut error_msg: Option<String> = None;

    loop {
        let result = tick(
            &mut terminal,
            &async_runtime,
            client_state.clone(),
            &config,
            &http_client
        );

        if let Err(error) = result {
            // TODO proper error handling?
            error_msg = Some(String::from("Something went wrong in tick"))
        }

        if client_state.lock().unwrap().status == Status::ReadyToExit {
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

    if let Some(error) = error_msg {
        eprintln!("{error}")
    }

    Ok(())
}

fn tick<B>(
    terminal: &mut Terminal<B>,
    async_runtime: &Runtime,
    client_state: Arc<Mutex<ClientState>>,
    config: &Config,
    http_client: &Client
) -> Result<(), Box<dyn Error>> where B : Backend {
    async_runtime.block_on(async {
        process_inputs(client_state.clone())?;
        process_state(client_state.clone(), config, http_client).await
    })?;
    render(terminal, client_state)?;

    Ok(())
}