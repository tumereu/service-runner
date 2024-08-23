use std::collections::{HashMap};
use std::thread::JoinHandle;
use crate::config::Config;
use crate::models::{CompileStatus, Dependency, OutputKey, OutputStore, Profile, RequiredState, RunStatus, Service, ServiceAction, ServiceStatus};
use crate::runner::file_watcher::FileWatcherState;
use crate::ui::UIState;

pub struct SystemState {
    pub current_profile: Option<Profile>,
    pub service_statuses: HashMap<String, ServiceStatus>,
    pub output_store: OutputStore,
    pub ui: UIState,
    pub config: Config,
    pub should_exit: bool,
    pub active_threads: Vec<(String, JoinHandle<()>)>,
    pub file_watchers: Option<FileWatcherState>
}

impl SystemState {
    pub fn new(config: Config) -> SystemState {
        SystemState {
            should_exit: false,
            current_profile: None,
            service_statuses: HashMap::new(),
            ui: UIState::new(),
            output_store: OutputStore::new(),
            active_threads: Vec::new(),
            file_watchers: None,
            config,
        }
    }

    pub fn get_profile_name(&self) -> Option<&str> {
        self.current_profile.as_ref().map(|profile| profile.name.as_str())
    }

    pub fn get_service(&self, service_name: &str) -> Option<&Service> {
        self.current_profile
            .as_ref()
            .and_then(|profile| {
                profile
                    .services
                    .iter()
                    .find(|service| service.name == service_name)
            })
    }

    pub fn iter_services(&self) -> impl Iterator<Item = &Service> {
        self.current_profile
            .iter()
            .flat_map(|profile| profile.services.iter())
    }

    pub fn iter_services_with_statuses(&self) -> impl Iterator<Item = (&Service, &ServiceStatus)> {
        self.current_profile
            .iter()
            .flat_map(|profile| profile.services.iter())
            .map(|service| {
                (service, self.get_service_status(&service.name).unwrap())
            })
    }

    pub fn get_service_status(&self, service_name: &str) -> Option<&ServiceStatus> {
        self.service_statuses.get(service_name)
    }

    pub fn is_satisfied(&self, dep: &Dependency) -> bool {
        self.get_service_status(&dep.service)
            .map(|status| {
                match dep.requirement {
                    RequiredState::Running => match (&status.run_status, &status.action) {
                        // The service must be running without any incoming changes
                        (RunStatus::Healthy, ServiceAction::None) => true,
                        (
                            RunStatus::Healthy,
                            ServiceAction::Restart | ServiceAction::Recompile
                        ) => false,
                        (RunStatus::Running | RunStatus::Failed | RunStatus::Stopped, _) => false,
                    },
                    RequiredState::Compiled => match (&status.compile_status, &status.action) {
                        (
                            CompileStatus::FullyCompiled,
                            ServiceAction::Restart | ServiceAction::None,
                        ) => true,
                        (_, ServiceAction::Recompile) => false,
                        (
                            CompileStatus::None
                            | CompileStatus::Compiling(_)
                            | CompileStatus::PartiallyCompiled(_)
                            | CompileStatus::Failed,
                            _,
                        ) => false,
                    },
                }
            })
            .unwrap_or(true)
    }

    /// TODO is this necessary anymore?
    pub fn update_state<F>(&mut self, update: F)
    where
        F: FnOnce(&mut SystemState),
    {
        update(self);
    }

    pub fn update_service_status<F>(&mut self, service: &str, update: F)
    where
        F: FnOnce(&mut ServiceStatus),
    {
        self.update_state(move |state| {
            let status = state.service_statuses.get_mut(service).unwrap();
            update(status);
        });
    }

    pub fn update_all_statuses<F>(&mut self, update: F)
    where
        F: Fn(&Service, &mut ServiceStatus),
    {
        self.update_state(move |state| {
            state.current_profile.as_ref()
                .iter()
                .flat_map(|profile| &profile.services)
                .for_each(|service| {
                        let status = state.service_statuses.get_mut(&service.name).unwrap();
                        update(service, status);

                        // Remove impossible configurations
                        if status.action == ServiceAction::Recompile && service.compile.is_none() {
                            status.action = ServiceAction::None;
                        } else if status.action == ServiceAction::Restart && service.run.is_none() {
                            status.action = ServiceAction::None;
                        }
                });
        });
    }

    pub fn add_output(&mut self, key: &OutputKey, line: String) {
        self.output_store.add_output(key, line);
    }
}
