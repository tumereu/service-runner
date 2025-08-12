use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::{env, error::Error, io::stdout, process, thread, time::Duration};

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::{debug, error, info, LevelFilter};
use ratatui::{backend::CrosstermBackend, Terminal};
use ::ui::{ComponentRenderer, UIError, UIResult};
use config::read_config;

use crate::input::process_inputs;
use crate::models::Action::ActivateProfile;
use crate::models::Profile;
use crate::runner::automation::start_automation_processor;
use crate::runner::file_watcher::start_file_watcher;
use crate::runner::process_action::process_action;
use crate::runner::rhai::RhaiExecutor;
use crate::runner::service_worker::ServiceWorker;
use crate::system_state::SystemState;
use crate::ui::{ViewRoot};

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

    let rhai_executor = Arc::new(RhaiExecutor::new(state_arc.clone()));
    let service_worker = Arc::new(ServiceWorker::new(state_arc.clone(), rhai_executor.clone()));

    let mut handles = vec![
        ("file-watcher".into(), start_file_watcher(state_arc.clone())),
        ("automation-processor".into(), start_automation_processor(state_arc.clone())),
        ("rhai-executor".into(), rhai_executor.start()),
        ("service-worker".into(), service_worker.start()),
    ];

    state_arc.lock().unwrap().active_threads.append(&mut handles);

    let join_threads = {
        let state_arc = state_arc.clone();
        thread::spawn(move || {
            let mut last_print = Instant::now();

            loop {
                {
                    let mut state = state_arc.lock().unwrap();
                    if state.should_exit && state.active_threads.is_empty() {
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
                            "System is trying to exit"
                        } else {
                            "System running normally"
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

    // Check for autolaunched profile
    {
        let mut system = state_arc.lock().unwrap();

        if let Some(autolaunch_profile) = &system.config.settings.autolaunch_profile {
            let selection = system.config.profiles.iter()
                .find(|profile| &profile.id == autolaunch_profile)
                .expect(&format!("Autolaunch profile with name '{}' not found", autolaunch_profile));

            let action = ActivateProfile(Profile::new(selection.clone(), &system.config.services));
            process_action(&mut system, action);
        }
    }

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut renderer = ComponentRenderer::new();
    let mut ui_result: UIResult<()> = Ok(());

    loop {
        process_inputs(state_arc.clone());

        match renderer.render_root(
            &mut terminal,
            ViewRoot {
                state: state_arc.clone(),
            }
        ) {
            Ok(_) => {},
            Err(error) => {
                error!("Encountered an unexpected exception during render(): {error:?}");
                ui_result = Err(error);
                break;
            }
        }

        if state_arc.lock().unwrap().should_exit {
            break;
        } else {
            thread::sleep(Duration::from_millis(10));
        }
    }

    state_arc.lock().unwrap().should_exit = true;
    service_worker.stop();
    rhai_executor.stop();

    match join_threads.join() {
        Ok(_) => info!("Threads joined successfully"),
        Err(error) => error!("Error when joining threads: {error:?}")
    }

    // Clear terminal and restore normal mode
    terminal.clear()?;
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
    )?;
    terminal.show_cursor()?;

    // If there were errors with the UI, panic at the very end after cleaning up the terminal
    match ui_result {
        Ok(_) => {},
        Err(error) => panic!("Unexpected error in UI rendering: {error}"),
    }

    Ok(())
}
