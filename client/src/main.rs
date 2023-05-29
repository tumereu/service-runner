use std::sync::{Arc, Mutex};
use std::{env, error::Error, io::stdout, thread, time::Duration};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};

use shared::config::read_config;
use shared::dbg_println;

use crate::client_state::{ClientState, ClientStatus};
use crate::connection::{connect_to_server, start_broadcast_processor};
use crate::input::process_inputs;
use crate::ui::render;

mod client_state;
mod connection;
mod input;
mod ui;

fn main() -> Result<(), Box<dyn Error>> {
    let config_dir: String = env::args()
        .collect::<Vec<String>>()
        .get(1)
        .ok_or("Specify the configuration directory in order to run the app")?
        .clone();

    let state = Arc::new(Mutex::new(ClientState::new(read_config(&config_dir)?)));
    let num_profiles = state.lock().unwrap().config.profiles.len();
    let num_services = state.lock().unwrap().config.services.len();

    dbg_println!(
        "Loaded configuration with {num_profiles} profile(s) and {num_services} service(s)"
    );

    enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    render(&mut terminal, state.clone())?;
    let broadcast_thread = start_broadcast_processor(state.clone());
    let stream_thread = connect_to_server(state.clone())?;

    loop {
        process_inputs(state.clone())?;
        render(&mut terminal, state.clone())?;

        if stream_thread.is_finished() {
            dbg_println!("Connection thread finished, marking client for exit");
            state.lock().unwrap().status = ClientStatus::Exiting;
        }

        if state.lock().unwrap().status == ClientStatus::Exiting {
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
    )?;
    terminal.show_cursor()?;

    stream_thread.join().ok();
    broadcast_thread.join().ok();

    Ok(())
}
