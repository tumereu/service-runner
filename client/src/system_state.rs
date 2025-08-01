use std::collections::HashMap;
use crate::config::{Block, Config};
use crate::models::{GetBlock, OutputKey, OutputStore, Profile, Service};
use crate::runner::file_watcher::FileWatcherState;
use crate::ui::UIState;
use std::thread::JoinHandle;
use crate::runner::service_worker::{AsyncOperationHandle, ProcessWrapper};

pub struct SystemState {
    pub current_profile: Option<Profile>,
    pub output_store: OutputStore,
    pub ui: UIState,
    pub config: Config,
    pub should_exit: bool,
    pub active_threads: Vec<(String, JoinHandle<()>)>,
    pub file_watchers: Option<FileWatcherState>,
    block_processes: HashMap<(String, String), AsyncOperationHandle>,
}

impl SystemState {
    pub fn new(config: Config) -> SystemState {
        SystemState {
            should_exit: false,
            current_profile: None,
            ui: UIState::new(),
            output_store: OutputStore::new(),
            active_threads: Vec::new(),
            file_watchers: None,
            block_processes: HashMap::new(),
            config,
        }
    }

    pub fn get_profile_name(&self) -> Option<&str> {
        self.current_profile.as_ref().map(|profile| profile.definition.id.as_str())
    }

    pub fn get_block_operation(&self, service_id: &str, block_id: &str) -> Option<&AsyncOperationHandle> {
        self.block_processes.get(&(service_id.to_owned(), block_id.to_owned()))
    }

    pub fn set_block_operation(&mut self, service_id: &str, block_id: &str, process: Option<AsyncOperationHandle>) {
        let key = (service_id.to_owned(), block_id.to_owned());
        
        match process {
            Some(process) => self.block_processes.insert(key, process),
            None => self.block_processes.remove(&key),
        };
    }

    pub fn get_service(&self, service_id: &str) -> Option<&Service> {
        self.current_profile
            .as_ref()
            .and_then(|profile| {
                profile
                    .services
                    .iter()
                    .find(|service| {
                        service.definition.id == service_id
                    })
            })
    }

    pub fn get_service_block(&self, service_id: &str, block_id: &str) -> Option<&Block> {
        self.get_service(service_id)
            .and_then(|service| service.get_block(block_id))
    }

    pub fn iter_services(&self) -> impl Iterator<Item = &Service> {
        self.current_profile
            .iter()
            .flat_map(|profile| profile.services.iter())
    }

    /*
    FIXME if needed
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
        i
}

     */

    pub fn update_service<F>(&mut self, service_id: &str, update: F)
    where
        F: FnOnce(&mut Service),
    {
        self.update_state(move |state| {
            let service_option = state.current_profile.as_mut()
                .and_then(|profile| {
                    profile.services.iter_mut()
                        .find(|service| service.definition.id == service_id)
                });

            if let Some(service) = service_option {
                update(service);
            }
        });
    }

    /// TODO is this necessary anymore?
    pub fn update_state<F>(&mut self, update: F)
    where
        F: FnOnce(&mut SystemState),
    {
        update(self);
    }

    pub fn add_output(&mut self, key: &OutputKey, line: String) {
        self.output_store.add_output(key, line);
    }
}
