use std::{env, error::Error, io::stdout, thread, time::Duration};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::CrosstermBackend,
    Terminal
};
use tui::backend::Backend;

use shared::config::{Config, read_config};
use shared::message::{Action, MessageTransmitter};

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

    let state = Arc::new(ClientState::new(read_config(&config_dir)?));

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

    connect_to_server(state.clone())?;

    loop {
        process_inputs(state.clone())?;
        render(&mut terminal, state.clone())?;

        if *state.status.lock().unwrap() == Status::Exiting {
            break;
        } else {
            thread::sleep(Duration::from_millis(10));
        }
    }

    disconnect_from_server(state.clone())?;

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

pub fn connect_to_server(state: Arc<ClientState>) -> Result<(), String> {
    let port = state.config.server.port;

    fn open_stream(port: u16) -> Option<TcpStream> {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        if let Ok(stream) = TcpStream::connect_timeout(&addr, Duration::from_millis(1000)) {
            Some(stream)
        } else {
            None
        }
    }

    let stream = open_stream(port);
    let stream = if stream.is_none() {
        Command::new(&state.config.server.executable)
            .arg(&state.config.conf_dir)
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

        open_stream(port)
    } else {
        stream
    };

    if stream.is_none() {
        Err(format!("Could not connect to server on port {port}"))
    } else {
        *state.stream.lock().unwrap() = stream;
        Ok(())
    }
}

pub fn disconnect_from_server(state: Arc<ClientState>) -> std::io::Result<()> {
    let mut stream = state.stream.lock().unwrap();

    if let Some(ref mut stream) = *stream {
        if state.config.server.daemon {
            stream.send(Action::Shutdown)?;
        }

        stream.shutdown(Shutdown::Both)?;
    }

    Ok(())
}