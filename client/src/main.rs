use std::sync::{Arc, RwLock};
use std::time::Instant;
use std::{env, error::Error, io::stdout, process, thread, time::Duration};

use config::read_config;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::{debug, error, info, LevelFilter};
use ratatui::{backend::CrosstermBackend, Terminal};
use ::ui::input::collect_input_events;
use ::ui::{ComponentRenderer, UIResult};

use crate::runner::file_watcher::FileWatcher;
use runner::scripting::executor::ScriptExecutor;
use crate::runner::service_worker::ServiceWorker;
use crate::system_state::SystemState;
use crate::ui::inputs::RegisterKeybinds;
use crate::ui::theming::RegisterTheme;
use crate::ui::ViewRoot;

mod system_state;
mod ui;
mod models;
mod runner;
mod utils;
pub mod config;

fn main() -> Result<(), Box<dyn Error>> {
    let config_dir: String = env::args()
        .collect::<Vec<String>>()
        .get(1)
        .ok_or("Specify the configuration directory in order to run the app")?
        .clone();
    let config = read_config(&config_dir);

    if let Err(error) = &config {
        println!("Error in configurations: {error}");
        process::exit(1);
    }
    let config = config?;
    
    simple_logging::log_to_file(
        config.settings.log_file.clone().unwrap_or("service_runner.log".to_string()),
        LevelFilter::Debug,
    )?;

    let system_state = Arc::new(RwLock::new(SystemState::new(config)));
    let num_profiles = system_state.read().unwrap().config.profiles.len();
    let num_services = system_state.read().unwrap().config.services.len();

    info!(
        "Loaded configuration with {num_profiles} profile(s) and {num_services} service(s)"
    );

    let rhai_executor = Arc::new(ScriptExecutor::new(system_state.clone()));
    let service_worker = Arc::new(ServiceWorker::new(system_state.clone(), rhai_executor.clone()));
    let file_watcher = Arc::new(FileWatcher::new(system_state.clone()));

    let mut handles = vec![
        ("file-watcher".into(), file_watcher.start()),
        ("rhai-executor".into(), rhai_executor.start()),
        ("service-worker".into(), service_worker.start()),
    ];

    system_state.write().unwrap().active_threads.append(&mut handles);

    let join_threads = {
        let state_arc = system_state.clone();
        thread::spawn(move || {
            let mut last_print = Instant::now();

            loop {
                {
                    let mut state = state_arc.write().unwrap();
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
        info!("Checking for autolaunched profile");
        let mut system = system_state.write().unwrap();

        let autolaunch_profile = if let Some(autolaunch_profile) = &system.config.settings.autolaunch_profile {
            let selection = system.config.profiles.iter()
                .find(|profile| &profile.id == autolaunch_profile)
                .expect(&format!("Autolaunch profile with name '{}' not found", autolaunch_profile))
                .id.clone();

            Some(selection)
        } else {
            None
        };

        if let Some(selection) = autolaunch_profile {
            info!("Autolaunching profile: {}", selection);
            system.select_profile(&selection);
        }
    }

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut renderer = ComponentRenderer::new();
    renderer.assign_default_attributes();
    renderer.register_theme(&system_state.read().unwrap().config.settings.theme);
    renderer.register_keybinds(&system_state.read().unwrap().config.settings.keybinds);

    let mut ui_result: UIResult<()> = Ok(());

    terminal.clear()?;
    loop {
        let input_events = collect_input_events();
        renderer.send_input_signals(input_events);

        match renderer.render_root(
            &mut terminal,
            ViewRoot {
                system_state: &mut system_state.write().unwrap(),
            },
        ) {
            Ok(_) => {}
            Err(error) => {
                error!("Encountered an unexpected exception during render(): {error:?}");
                ui_result = Err(error);
                break;
            }
        }

        if system_state.read().unwrap().should_exit {
            break;
        } else {
            thread::sleep(Duration::from_millis(50));
        }
    }

    system_state.write().unwrap().should_exit = true;
    file_watcher.stop();
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
        Ok(_) => {}
        Err(error) => panic!("Unexpected error in UI rendering: {error}"),
    }

    Ok(())
}
