use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use log::{error, info, trace};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use crate::config::AutomationTrigger;
use crate::runner::automation::{enqueue_automation, process_pending_automations};
use crate::system_state::SystemState;


pub struct FileWatcherState {
    pub profile_name: String,
    pub watchers: Vec<RecommendedWatcher>,
    pub latest_events: HashMap<String, Instant>,
    pub latest_recompiles: HashMap<String, Instant>,
}

pub fn start_file_watcher(system_arc: Arc<Mutex<SystemState>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while !system_arc.lock().unwrap().should_exit {
            let rebuild_watchers = {
                let system = system_arc.lock().unwrap();
                match (system.get_profile_name(), &system.file_watchers) {
                    (None, None) => false,
                    (None, Some(_)) => true,
                    (Some(_), None) => true,
                    (Some(profile), Some(FileWatcherState { profile_name, .. })) => profile != profile_name,
                }
            };

            if rebuild_watchers {
                info!("Rebuilding file watchers due to a change in profile");
                setup_watchers(system_arc.clone());
            }

            thread::sleep(Duration::from_millis(100))
        }

        // Dropping the file watcher state should automatically clean up the created watchers
        system_arc.lock().unwrap().file_watchers = None;
    })
}


fn setup_watchers(system_arc: Arc<Mutex<SystemState>>) {
    let mut system = system_arc.lock().unwrap();

    let new_watchers = if let Some(profile_name) = system.get_profile_name() {
        let watchers: Vec<RecommendedWatcher> = system.iter_services()
            .flat_map(|service| {
                service.definition.automation.iter()
                    .flat_map(|automation_entry| {
                        match &automation_entry.trigger {
                            AutomationTrigger::ModifiedFile { paths } => {
                                Some((
                                    service.definition.name.clone(),
                                    service.definition.dir.clone(),
                                    automation_entry.clone(),
                                    paths.clone()
                                ))
                            },
                            _ => None
                        }
                    })
            })
            .filter_map(|(service_name, work_dir, automation_entry, watch_paths)| {
                info!("Creating a watcher for service {service_name} with paths {watch_paths:?}");

                let watcher = {
                    let system_arc = system_arc.clone();
                    let service_name = service_name.clone();

                    notify::recommended_watcher(move |res| {
                        match res {
                            Ok(event) => {
                                trace!("Received filesystem event for service {service_name}: {event:?}");
                                let mut system = system_arc.lock().unwrap();

                                enqueue_automation(&mut system, &service_name, &automation_entry);
                                process_pending_automations(&mut system);
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
                        watch_path.push(Path::new(&work_dir));
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
            })
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

    system.file_watchers = new_watchers;
}