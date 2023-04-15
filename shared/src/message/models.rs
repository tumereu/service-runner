use std::collections::{HashMap, VecDeque};


use serde::{Deserialize, Serialize};

use crate::config::{
    ExecutableEntry as ConfigExecutableEntry,
    Profile as ConfigProfile,
    Service as ConfigService
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Service {
    Compilable {
        name: String,
        dir: String,
        compile: Vec<ExecutableEntry>,
        run: Vec<ExecutableEntry>,
        reset: Vec<ExecutableEntry>,
    }
}
impl Service {
    pub fn name(&self) -> &String {
        match self {
            Service::Compilable { name, .. } => &name,
        }
    }
}
impl From<ConfigService> for Service {
    fn from(value: ConfigService) -> Self {
        match value {
            ConfigService::Compilable { name, dir, compile, run, reset } => {
                Service::Compilable {
                    name,
                    dir,
                    compile: compile.into_iter().map(|ex| ex.into()).collect(),
                    run: run.into_iter().map(|ex| ex.into()).collect(),
                    reset: reset.into_iter().map(|ex| ex.into()).collect(),
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExecutableEntry {
    pub executable: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}
impl From<ConfigExecutableEntry> for ExecutableEntry {
    fn from(value: ConfigExecutableEntry) -> Self {
        ExecutableEntry {
            executable: value.executable,
            args: value.args,
            env: value.env,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Profile {
    pub name: String,
    pub services: Vec<Service>
}
impl Profile {
    pub fn new(profile: &ConfigProfile, all_services: &Vec<ConfigService>) -> Profile {
        let services: Vec<Service> = all_services.iter()
            .filter(|service| profile.includes(service))
            .map(|service| service.clone().into())
            .collect();

        Profile {
            name: profile.name.clone(),
            services
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceStatus {
    pub should_run: bool,
    pub auto_recompile: bool,
    pub needs_compile: bool,
    pub compile_status: CompileStatus,
    pub is_running: bool,
    pub show_output: bool,
}
impl ServiceStatus {
    pub fn from(_profile: &Profile, _service: &Service) -> ServiceStatus {
        ServiceStatus {
            should_run: true,
            auto_recompile: true,
            needs_compile: true,
            compile_status: CompileStatus::None,
            is_running: false,
            show_output: true,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CompileStatus {
    None,
    Compiling(usize),
    Compiled(usize)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OutputStore {
    pub outputs: HashMap<OutputKey, VecDeque<OutputLine>>
}
impl OutputStore {
    pub fn new() -> Self {
        OutputStore {
            outputs: HashMap::new()
        }
    }

    pub fn add_output(&mut self, key: &OutputKey, line: OutputLine) {
        if !self.outputs.contains_key(key) {
            self.outputs.insert(key.clone(), VecDeque::new());
        }
        let deque = self.outputs.get_mut(key).unwrap();
        deque.push_back(line);
        // TODO move to a config field
        if deque.len() > 8096 {
            deque.pop_front();
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct OutputKey {
    pub name: String,
    pub service_ref: String,
    pub kind: OutputKind
}
impl OutputKey {
    pub fn new(name: String, service_ref: String, kind: OutputKind) -> Self {
        OutputKey {
            name,
            service_ref,
            kind
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum OutputKind {
    Compile,
    Run
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OutputLine {
    pub value: String,
    pub timestamp: u128
}