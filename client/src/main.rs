use std::{env, error::Error, fs::File, io::{BufReader, Result as IOResult, stdout}, path::Path, task, thread, time::Duration};
use std::process::{Command, Stdio};
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
use tui::backend::Backend;
use tui::style::{Color, Style};

use shared::config::{Config, read_config};

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

    let config = Arc::new(read_config(&config_dir)?);

    let mut client_state = Arc::new(Mutex::new(ClientState::new()));

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

    {
        let config = config.clone();
        let client_state = client_state.clone();

        thread::spawn(move || {
            loop {
                process_inputs(client_state.clone(), config.clone())?;

                if client_state.lock().unwrap().status == Status::Exiting {
                    break;
                } else {
                    thread::sleep(Duration::from_millis(10));
                }
            }

            Ok::<(), String>(())
        });
    }

    connect_to_server(client_state.clone(), config.clone());

    loop {
        let result = tick(
            &mut terminal,
            client_state.clone(),
            &config,
        );

        if let Err(_) = result {
            // TODO proper error handling?
            error_msg = Some(String::from("Something went wrong in tick"))
        }

        if client_state.lock().unwrap().status == Status::Exiting {
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

pub fn connect_to_server(state: Arc<Mutex<ClientState>>, config: Arc<Config>) -> Result<(), String> {
    let port = config.server.port;
    let status = state.lock().unwrap().status;

    match status {
        Status::CheckingServerStatus => {
            // TODO open socket
        }
        Status::StartingServer => {
            Command::new(config.server.executable.clone())
                .arg(&config.conf_dir)
                .current_dir(env::current_dir().map_err(|err| {
                    let msg = err.to_string();
                    format!("Failed to read current workdir: {msg}")
                })?)
                .stdout(Stdio::null())
                .stdin(Stdio::null())
                .spawn()
                .map_err(|err| {
                    let msg = err.to_string();
                    format!("Failed to spawn server process: {msg}")
                })?;

            let mut state = state.lock().unwrap();
            state.status = Status::CheckingServerStatus
        }
        Status::Ready => {
        }
        _ => {

        }
    }

    Ok(())
}

fn tick<B>(
    terminal: &mut Terminal<B>,
    client_state: Arc<Mutex<ClientState>>,
    config: &Config,
) -> Result<(), Box<dyn Error>> where B : Backend {
    render(terminal, client_state, &config)?;

    Ok(())
}