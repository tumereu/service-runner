use std::{env, error::Error, io::stdout, thread, time::Duration};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::CrosstermBackend,
    Terminal
};

use shared::config::{Config, read_config};
use shared::message::{Action, MessageTransmitter};

use crate::client_state::{ClientState, Status};
use crate::connection::connect_to_server;
use crate::input::process_inputs;
use crate::ui::render;

mod client_state;
mod ui;
mod input;
mod connection;

fn main() -> Result<(), Box<dyn Error>> {
    let config_dir: String = env::args().collect::<Vec<String>>()
        .get(1)
        .ok_or("Specify the configuration directory in order to run the app")?
        .clone();

    let state = Arc::new(Mutex::new(ClientState::new(read_config(&config_dir)?)));

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

    render(&mut terminal, state.clone())?;
    let stream_thread = connect_to_server(state.clone())?;

    loop {
        process_inputs(state.clone())?;
        render(&mut terminal, state.clone())?;

        if state.lock().unwrap().status == Status::Exiting {
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

    stream_thread.join().expect("Could not join the stream-handler")?;

    if let Some(error) = error_msg {
        eprintln!("{error}")
    }

    Ok(())
}