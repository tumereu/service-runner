use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use log::{error, info, trace};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use crate::config::{AutomationTrigger, ServiceId};
use crate::models::Automation;
use crate::runner::automation::{enqueue_automation, process_pending_automations};
use crate::system_state::SystemState;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
struct AutomationKey {
    automation_name: String,
    service_id: Option<ServiceId>,
}

pub struct FileWatcher {
    state: Arc<RwLock<SystemState>>,
    keep_alive: Arc<RwLock<bool>>,
    watchers: Arc<RwLock<HashMap<AutomationKey, RecommendedWatcher>>>,
}
impl FileWatcher {
    pub fn new(state: Arc<RwLock<SystemState>>) -> Self {
        Self {
            state,
            keep_alive: Arc::new(RwLock::new(true)),
            watchers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn start(&self) -> JoinHandle<()> {
        let keep_alive = self.keep_alive.clone();
        let watchers = self.watchers.clone();
        let state = self.state.clone();

        thread::spawn(move || {
            while *keep_alive.read().unwrap() {
                let mut watchers = watchers.write().unwrap();
                let state = state.read().unwrap();

                state
                    .current_profile
                    .iter()
                    .flat_map(|profile| profile.automations.iter())
                    .for_each(|automation| {});

                thread::sleep(Duration::from_millis(100))
            }
        })
    }

    fn handle_automation(
        watchers: &mut HashMap<AutomationKey, RecommendedWatcher>,
        automation: &Automation,
        service_id: Option<ServiceId>,
    ) {
        let key = AutomationKey {
            automation_name: automation.definition.name.clone(),
            service_id
        };
        let is_watching = watchers.contains_key(&key);
        
        
    }
}
pub fn start_file_watcher(system_arc: Arc<RwLock<SystemState>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while !system_arc.read().unwrap().should_exit {
            let rebuild_watchers = {
                let system = system_arc.read().unwrap();
                match (system.get_profile_name(), &system.file_watchers) {
                    (None, None) => false,
                    (None, Some(_)) => true,
                    (Some(_), None) => true,
                    (Some(profile), Some(FileWatcherState { profile_name, .. })) => {
                        profile != profile_name
                    }
                }
            };

            if rebuild_watchers {
                info!("Rebuilding file watchers due to a change in profile");
                setup_watchers(system_arc.clone());
            }

            thread::sleep(Duration::from_millis(100))
        }

        // Dropping the file watcher state should automatically clean up the created watchers
        system_arc.write().unwrap().file_watchers = None;
    })
}

fn setup_watchers(system_arc: Arc<RwLock<SystemState>>) {
    let mut system = system_arc.write().unwrap();

    let new_watchers = if let Some(profile_name) = system.get_profile_name() {
        let watchers: Vec<RecommendedWatcher> = system.iter_services()
            .flat_map(|service| {
                service.definition.automation.iter()
                    .flat_map(|automation_entry| {
                        match &automation_entry.trigger {
                            AutomationTrigger::FileModified { paths } => {
                                Some((
                                    service.definition.id.clone(),
                                    service.definition.workdir.clone(),
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
                                let mut system = system_arc.write().unwrap();

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
        }
        .into()
    } else {
        None
    };

    system.file_watchers = new_watchers;
}
