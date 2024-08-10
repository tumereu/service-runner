use std::sync::{Arc, Mutex};
use std::{env, error::Error, io::stdout, process, thread, time::Duration};
use std::time::Instant;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};

use config::read_config;
use log::{debug, info, LevelFilter};

use crate::system_state::{SystemState};
use crate::input::process_inputs;
use crate::runner::file_watcher::start_file_watcher;
use crate::ui::render;
use crate::runner::service_worker::start_service_worker;

mod system_state;
mod input;
mod ui;
mod models;
mod runner;
mod utils;
pub mod config;

fn main() -> Result<(), Box<dyn Error>> {
    simple_logging::log_to_file("service_runner.log", LevelFilter::Debug)?;

    let config_dir: String = env::args()
        .collect::<Vec<String>>()
        .get(1)
        .ok_or("Specify the configuration directory in order to run the app")?
        .clone();
    let config = read_config(&config_dir);

    if let Err(error) = &config {
        let filename = &error.filename;
        let message = &error.user_message;

        println!("Error: failed to parse configuration file {filename}: {message}");
        process::exit(1);
    }

    let state_arc = Arc::new(Mutex::new(SystemState::new(config.unwrap())));
    let num_profiles = state_arc.lock().unwrap().config.profiles.len();
    let num_services = state_arc.lock().unwrap().config.services.len();

    info!(
        "Loaded configuration with {num_profiles} profile(s) and {num_services} service(s)"
    );

    enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    render(&mut terminal, state_arc.clone())?;

    let mut handles = vec![
        ("service-worker".into(), start_service_worker(state_arc.clone())),
        ("file-watcher".into(), start_file_watcher(state_arc.clone())),
    ];

    state_arc.lock().unwrap().active_threads.append(&mut handles);

    let join_threads = {
        let state_arc = state_arc.clone();
        thread::spawn(move || {
            let mut last_print = Instant::now();

            loop {
                {
                    let mut state = state_arc.lock().unwrap();
                    if state.should_exit && state.active_threads.len() == 0 {
                        break;
                    }

                    state.active_threads.retain(|(_, thread)| !thread.is_finished());

                    let print_delay = if state.should_exit {
                        Duration::from_millis(1000)
                    } else {
                        Duration::from_millis(60_000)
                    };

                    if Instant::now().duration_since(last_print) >= print_delay {
                        let status = if state.should_exit {
                            "Server is trying to exit"
                        } else {
                            "Server running normally"
                        };

                        let thread_count = state.active_threads.len();
                        let threads = state.active_threads.iter()
                            .map(|(name, _)| name.clone())
                            .collect::<Vec<String>>()
                            .join(", ");

                        debug!("{status}. Active threads ({thread_count} total): {threads}");
                        last_print = Instant::now();
                    }
                }

                thread::sleep(Duration::from_millis(10));
            }
        })
    };

    loop {
        process_inputs(state_arc.clone())?;
        render(&mut terminal, state_arc.clone())?;

        if state_arc.lock().unwrap().should_exit {
            break;
        } else {
            thread::sleep(Duration::from_millis(10));
        }
    }

    join_threads.join().unwrap();

    // Clear terminal and restore normal mode
    terminal.clear()?;
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
    )?;
    terminal.show_cursor()?;

    Ok(())
}
