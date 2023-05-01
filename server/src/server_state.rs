use std::collections::{HashMap, VecDeque};
use std::thread::JoinHandle;
use std::time::Instant;
use notify::RecommendedWatcher;

use shared::message::models::{CompileStatus, Dependency, OutputKey, OutputStore, Profile, RequiredState, RunStatus, Service, ServiceAction, ServiceStatus};
use shared::message::{Action, Broadcast};
use shared::system_state::SystemState;

pub struct ServerState {
    pub created: Instant,
    pub actions_in: VecDeque<Action>,
    pub broadcasts_out: HashMap<u32, VecDeque<Broadcast>>,
    system_state: SystemState,
    pub output_store: OutputStore,
    pub active_threads: Vec<JoinHandle<()>>,
    pub file_watchers: Option<FileWatcherState>
}

pub struct FileWatcherState {
    pub profile_name: String,
    pub watchers: Vec<RecommendedWatcher>,
    pub latest_events: HashMap<String, Instant>,
    pub latest_recompiles: HashMap<String, Instant>,
}

impl ServerState {
    pub fn new() -> ServerState {
        ServerState {
            created: Instant::now(),
            actions_in: VecDeque::new(),
            broadcasts_out: HashMap::new(),
            system_state: SystemState::new(),
            output_store: OutputStore::new(),
            active_threads: Vec::new(),
            file_watchers: None,
        }
    }

    pub fn get_profile_name(&self) -> Option<&str> {
        self.system_state.current_profile.as_ref().map(|profile| profile.name.as_str())
    }

    pub fn get_state(&self) -> &SystemState {
        &self.system_state
    }

    pub fn get_service(&self, service_name: &str) -> Option<&Service> {
        self.system_state
            .current_profile
            .as_ref()
            .map(|profile| {
                profile
                    .services
                    .iter()
                    .find(|service| service.name == service_name)
            })
            .flatten()
    }

    pub fn iter_services(&self) -> impl Iterator<Item = &Service> {
        self.system_state.current_profile
            .iter()
            .flat_map(|profile| profile.services.iter())
    }

    pub fn get_service_status(&self, service_name: &str) -> Option<&ServiceStatus> {
        self.system_state.service_statuses.get(service_name)
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

    pub fn update_state<F>(&mut self, update: F)
    where
        F: FnOnce(&mut SystemState),
    {
        update(&mut self.system_state);
        let broadcast = Broadcast::State(self.system_state.clone());
        self.broadcast_all(broadcast);
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
                });
        });
    }

    pub fn broadcast_all(&mut self, broadcast: Broadcast) {
        self.broadcasts_out.iter_mut().for_each(|(_, queue)| {
            queue.push_back(broadcast.clone());
        });
    }

    pub fn broadcast_one(&mut self, client: u32, broadcast: Broadcast) {
        let queue = self.broadcasts_out.get_mut(&client);
        if let Some(queue) = queue {
            queue.push_back(broadcast);
        }
    }

    pub fn add_output(&mut self, key: &OutputKey, line: String) {
        let line = self.output_store.add_output(key, line).clone();
        self.broadcast_all(Broadcast::OutputLine(key.clone(), line));
    }
}
