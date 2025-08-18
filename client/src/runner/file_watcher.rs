use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use log::{error, info, trace};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use crate::config::{AutomationDefinitionId, AutomationTrigger, ServiceId};
use crate::models::{Automation, AutomationStatus};
use crate::system_state::SystemState;
use crate::utils::resolve_path;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
struct AutomationKey {
    def_id: AutomationDefinitionId,
    service_id: Option<ServiceId>,
}

pub struct FileWatcher {
    state: Arc<RwLock<SystemState>>,
    keep_alive: Arc<RwLock<bool>>,
    /// File watchers indexed by (automation_id, service_id)-keys, where service id is optional. The value is a Some
    /// containing a recommended watcher if the automation is about watching files and the file watcher has been created,
    /// a None if the automation has been processed and doesn't require file watchers. If the map doesn't contain a
    /// value for a key, then the automation either is not enabled or has not yet been processed.
    watchers: Arc<RwLock<HashMap<AutomationKey, Option<RecommendedWatcher>>>>,
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
                // Collect all automations in the whole profile
                let automation_ids: Vec<(AutomationDefinitionId, Option<ServiceId>)> = {
                    let state = state.read().unwrap();
                    state
                        .current_profile
                        .iter()
                        .flat_map(|profile| &profile.automations)
                        .map(|automation| (automation.definition.id.clone(), None))
                        .chain(
                            state
                                .current_profile
                                .iter()
                                .flat_map(|profile| &profile.services)
                                .flat_map(|service| {
                                    service.automations.iter().map(|automation| {
                                        (automation.definition.id.clone(), Some(service.definition.id.clone()))
                                    })
                                })
                        )
                        .collect()
                };

                let mut watchers = watchers.write().unwrap();
                for (automation_id, service_id) in automation_ids {
                    Self::handle_automation(
                        state.clone(),
                        &mut watchers,
                        &automation_id,
                        service_id,
                    )
                }

                thread::sleep(Duration::from_millis(100))
            }
        })
    }

    pub fn stop(&self) {
        *self.keep_alive.write().unwrap() = false;
    }

    fn handle_automation(
        system_state: Arc<RwLock<SystemState>>,
        watchers: &mut HashMap<AutomationKey, Option<RecommendedWatcher>>,
        automation_id: &AutomationDefinitionId,
        service_id: Option<ServiceId>,
    ) {
        let key = AutomationKey {
            def_id: automation_id.clone(),
            service_id: service_id.clone(),
        };
        let is_watching = watchers.contains_key(&key);
        let enabled = {
            match system_state.read().unwrap().query_automation(
                &automation_id,
                &service_id,
                |automation| automation.status.clone(),
            ) {
                None => false,
                Some(AutomationStatus::Disabled) => false,
                _ => true,
            }
        };

        if is_watching && !enabled {
            watchers.remove(&key);
        } else if !is_watching && enabled {
            // Resolve work directory. If the automation is for a service, use the service's work directory. If the
            // automation is for the whole profile, use the profile's work directory.
            let work_dir = service_id
                .as_ref()
                .and_then(|service_id| {
                    system_state
                        .read()
                        .unwrap()
                        .query_service(&service_id, |service| service.definition.workdir.clone())
                })
                .or_else(|| {
                    system_state
                        .read()
                        .unwrap()
                        .current_profile
                        .as_ref()
                        .map(|profile| profile.definition.workdir.clone())
                })
                .unwrap_or_default();

            // Resolve the paths to watch for this automation. Note that this may not necessarily even yield any
            // results, the profile could've changed asynchronously or the automation may not defined any paths to
            // watch.
            let watch_paths: Vec<PathBuf> = system_state
                .read()
                .unwrap()
                .query_automation(&automation_id, &service_id, |automation| {
                    automation
                        .definition
                        .triggers
                        .iter()
                        .filter_map(|trigger| match trigger {
                            AutomationTrigger::RhaiQuery { .. } => None,
                            AutomationTrigger::FileModified { file_modified } => {
                                Some(file_modified.clone())
                            }
                        })
                        .map(|watch_path| resolve_path(&work_dir, &watch_path))
                        .collect()
                })
                .unwrap_or_default();

            // If there's nothing to watch, we mark the automation as properly handled by inserting None into the map
            if watch_paths.is_empty() {
                watchers.insert(key, None);
            } else {
                Self::create_automation_watcher(
                    key,
                    system_state,
                    watch_paths,
                    watchers,
                    automation_id,
                    service_id,
                );
            }
        }
    }

    fn create_automation_watcher(
        key: AutomationKey,
        system_state: Arc<RwLock<SystemState>>,
        watch_paths: Vec<PathBuf>,
        watchers: &mut HashMap<AutomationKey, Option<RecommendedWatcher>>,
        automation_id: &AutomationDefinitionId,
        service_id: Option<ServiceId>,
    ) {
        let watcher = {
            let system_state = system_state.clone();
            let automation_id = automation_id.clone();
            let service_id = service_id.clone();

            notify::recommended_watcher(move |res| match res {
                Ok(event) => {
                    trace!(
                        "Received filesystem event for automation {:?} (service: {:?}): {:?}",
                        automation_id,
                        service_id,
                        event,
                    );
                    let mut system = system_state.write().unwrap();

                    system.update_automation(&automation_id, &service_id, |automation| {
                        automation.last_triggered = Some(Instant::now());
                    });
                }
                Err(err) => error!(
                    "Error in file watcher event for automation {:?} (service: {:?}): {:?}",
                    automation_id, service_id, err,
                ),
            })
        };

        let successful = if let Ok(mut watcher) = watcher {
            let mut successful = true;
            for path in watch_paths {
                let watch_result = watcher.watch(&path, RecursiveMode::Recursive);
                if watch_result.is_err() {
                    error!(
                        "Failure when trying to watch path {path:?} for automation {automation_id:?} (service: {service_id:?}): {watch_result:?}"
                    );
                    successful = false;
                    break;
                }
            }

            watchers.insert(key, Some(watcher));
            successful
        } else {
            error!(
                "Failed to create a file watcher for automation {:?} (service: {:?})",
                automation_id, service_id
            );
            watchers.insert(key, None);
            false
        };

        let mut system = system_state.write().unwrap();
        system.update_automation(&automation_id, &service_id, |automation| {
            automation.status = match automation.status {
                AutomationStatus::Disabled => AutomationStatus::Disabled,
                AutomationStatus::Active if successful => AutomationStatus::Active,
                AutomationStatus::Active => AutomationStatus::Error,
                AutomationStatus::Error => AutomationStatus::Error,
            };
        });
    }
}
