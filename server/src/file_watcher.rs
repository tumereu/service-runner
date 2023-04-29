use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use shared::message::models::{AutoCompileTrigger, ServiceAction};
use shared::system_state::Status;
use itertools::Itertools;
use shared::dbg_println;
use crate::server_state::{FileWatcherState, ServerState};

pub fn start_file_watcher(server: Arc<Mutex<ServerState>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while server.lock().unwrap().get_state().status != Status::Exiting {
            let rebuild_watchers = {
                let mut server = server.lock().unwrap();
                match (server.get_profile_name(), &server.file_watchers) {
                    (None, None) => false,
                    (None, Some(_)) => true,
                    (Some(_), None) => true,
                    (Some(profile), Some(FileWatcherState { profile_name, .. })) => profile != profile_name,
                }
            };
            if rebuild_watchers {
                dbg_println!("Rebuilding file watchers due to a change in profile");
                setup_watchers(server.clone());
            }

            check_triggers(server.clone());

            thread::sleep(Duration::from_millis(100))
        }
    })
}

fn setup_watchers(server: Arc<Mutex<ServerState>>) {
    let mut server_state = server.lock().unwrap();
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
                dbg_println!("Creating a watcher for service {service_name} with paths {watch_paths:?}");
                let mut watcher = {
                    let server = server.clone();
                    let service_name = service_name.clone();
                    notify::recommended_watcher(move |res| {
                        match res {
                            Ok(event) => {
                                dbg_println!("Received filesystem event for service {service_name}: {event:?}");
                                server.lock().unwrap().file_watchers
                                    .iter_mut()
                                    .for_each(|watcher_state| {
                                        watcher_state.latest_events.insert(service_name.clone(), Instant::now());
                                    })
                            }
                            Err(err) => dbg_println!("Error in file watcher for service {service_name}: {err:?}"),
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
                            dbg_println!(
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
                    dbg_println!("Failed to create a file watcher for {service_name}: {watcher:?}");
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

fn check_triggers(server: Arc<Mutex<ServerState>>) {
    let triggered_services: Vec<String> = {
        server.lock().unwrap()
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

    let mut server = server.lock().unwrap();
    for service in triggered_services {
        server.update_service_status(&service, |service| {
            service.action = ServiceAction::Recompile;
        });
    }
}