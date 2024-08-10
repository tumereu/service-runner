use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use log::{debug, error, info};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use crate::models::action::models::{AutoCompileMode, AutoCompileTrigger, ServiceAction};
use crate::models::{AutoCompileMode, AutoCompileTrigger, ServiceAction};
use crate::models::runner_state::Status;
use crate::runner::file_watcher_state::{FileWatcherState, ServerState};
use crate::system_state::SystemState;


pub struct FileWatcherState {
    pub profile_name: String,
    pub watchers: Vec<RecommendedWatcher>,
    pub latest_events: HashMap<String, Instant>,
    pub latest_recompiles: HashMap<String, Instant>,
}

pub fn start_file_watcher(state: Arc<Mutex<SystemState>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while !state.lock().unwrap().should_exit {
            let rebuild_watchers = {
                let server = state.lock().unwrap();
                match (server.get_profile_name(), &server.file_watchers) {
                    (None, None) => false,
                    (None, Some(_)) => true,
                    (Some(_), None) => true,
                    (Some(profile), Some(FileWatcherState { profile_name, .. })) => profile != profile_name,
                }
            };
            if rebuild_watchers {
                info!("Rebuilding file watchers due to a change in profile");
                setup_watchers(state.clone());
            }

            check_triggers(state.clone());

            thread::sleep(Duration::from_millis(100))
        }
    })
}

fn setup_watchers(state: Arc<Mutex<SystemState>>) {
    let mut server_state = state.lock().unwrap();
    let new_watchers = if let Some(profile_name) = server_state.get_profile_name() {
        let watchers: Vec<RecommendedWatcher> = server_state.iter_services()
            .flat_map(|service| {
                service.autocompile.iter()
                    .flat_map(|autocompile| &autocompile.triggers)
                    .filter(|trigger| matches!(trigger, AutoCompileTrigger::ModifiedFile { .. }))
                    .map(|trigger| {
                        (
                            service.name.clone(),
                            service.dir.clone(),
                            match trigger {
                                AutoCompileTrigger::ModifiedFile { paths } => paths.clone(),
                                _ => panic!("The trigger must be a modified file in this section of code!")
                            }
                        )
                    })
            })
            .map(|(service_name, work_dir, watch_paths)| {
                info!("Creating a watcher for service {service_name} with paths {watch_paths:?}");
                let watcher = {
                    let server = state.clone();
                    let service_name = service_name.clone();
                    notify::recommended_watcher(move |res| {
                        match res {
                            Ok(event) => {
                                debug!("Received filesystem event for service {service_name}: {event:?}");
                                let mut server = server.lock().unwrap();
                                match server.get_service_status(&service_name).map(|status| status.auto_compile.clone()).flatten() {
                                    // For automatic compile, add the server into the events so that the recompile can
                                    // be triggered later, as the debounce interval passes
                                    Some(AutoCompileMode::Automatic) => {
                                        server.file_watchers
                                            .iter_mut()
                                            .for_each(|watcher_state| {
                                                watcher_state.latest_events.insert(service_name.clone(), Instant::now());
                                            })
                                    },
                                    // For triggered compile we just mark the service as having changes
                                    Some(AutoCompileMode::Custom) => {
                                        server.update_service_status(&service_name, |status| {
                                            status.has_uncompiled_changes = true;
                                        });
                                    }
                                    _ => {}
                                }
                            }
                            Err(err) => error!("Error in file watcher for service {service_name}: {err:?}"),
                        }
                    })
                };

                if let Ok(mut watcher) = watcher {
                    // TODO output some error to the Output-system in case there is a failure?
                    let mut successful = true;
                    for path in &watch_paths {
                        let mut watch_path = PathBuf::new();
                        if let Some(dir) = work_dir.as_ref() {
                            watch_path.push(Path::new(dir));
                        }
                        watch_path.push(Path::new(path));

                        let watch_result = watcher.watch(&watch_path, RecursiveMode::Recursive);
                        if watch_result.is_err() {
                            error!(
                                "Failure when trying to watch path {watch_path:?} for service {service_name}: {watch_result:?}"
                            );
                            successful = false;
                            break;
                        }
                    }

                    if successful {
                        Some(watcher)
                    } else {
                        None
                    }
                } else {
                    error!("Failed to create a file watcher for {service_name}: {watcher:?}");
                    None
                }
            }).flatten()
            .collect();

        FileWatcherState {
            profile_name: profile_name.to_string(),
            watchers,
            latest_events: HashMap::new(),
            latest_recompiles: HashMap::new(),
        }.into()
    } else {
        None
    };

    server_state.file_watchers = new_watchers;
}

fn check_triggers(state_arc: Arc<Mutex<SystemState>>) {
    let triggered_services: Vec<String> = {
        state_arc.lock().unwrap()
            .file_watchers
            .iter()
            .flat_map(|watcher_state| {
                watcher_state.latest_events
                    .iter()
                    // Filter events down to those that have occurred since the last triggered recompile
                    .filter(|(service, timestamp)| {
                        watcher_state.latest_recompiles.get(service.as_str())
                            .map(|recompile_timestamp| timestamp > &recompile_timestamp)
                            .unwrap_or(true)
                    })
                    // Debounce the remaining events
                    .filter(|(_, &timestamp)| {
                        // TODO move debounce time to a config somewhere?
                        Instant::now().duration_since(timestamp).as_millis() > 3000
                    })
            })
            .map(|(service, _)| service.clone())
            .collect()
    };

    let mut state = state_arc.lock().unwrap();
    for service in triggered_services {
        state.update_service_status(&service, |service| {
            service.action = ServiceAction::Recompile;
        });
    }
}