use std::collections::{HashMap, VecDeque};
use std::convert::Into;
use std::fmt::{Display, Formatter, Write};
use std::ptr::replace;
use std::str::Chars;
use serde::{Deserialize, Serialize};
use toml::value::Index;

use crate::config::{
    ExecutableEntry as ConfigExecutableEntry, Profile as ConfigProfile, Service as ConfigService, ScriptedRunConfig as ConfigScriptedRunConfig, HealthCheck as ConfigHealthCheck, Dependency as ConfigDependency, RequiredState as ConfigRequiredState, ScriptedCompileConfig as ConfigScriptedCompileConfig};
use crate::message::models::ServiceAction::{Recompile, Restart};
use crate::write_escaped_str;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Service {
    pub name: String,
    pub dir: Option<String>,
    pub compile: Option<CompileConfig>,
    pub run: Option<RunConfig>,
    pub reset: Vec<ExecutableEntry>,
}

impl From<ConfigService> for Service {
    fn from(value: ConfigService) -> Self {
        match value {
            ConfigService::Scripted { name, dir, compile, run, reset } => {
                Service {
                    name,
                    dir: dir.into(),
                    compile: compile.map(Into::into),
                    run: run.map(Into::into),
                    reset: reset.into_iter().map(Into::into).collect(),
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Dependency {
    pub service: String,
    pub requirement: RequiredState
}
impl From<ConfigDependency> for Dependency {
    fn from(value: ConfigDependency) -> Self {
        Dependency {
            service: value.service,
            requirement: value.require.into()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RequiredState {
    Compiled,
    Running
}
impl From<ConfigRequiredState> for RequiredState {
    fn from(value: ConfigRequiredState) -> Self {
        match value {
            ConfigRequiredState::Compiled => RequiredState::Compiled,
            ConfigRequiredState::Running => RequiredState::Running
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CompileConfig {
    pub commands: Vec<ExecutableEntry>,
    pub dependencies: Vec<Dependency>,
}
impl From<ConfigScriptedCompileConfig> for CompileConfig {
    fn from(value: ConfigScriptedCompileConfig) -> Self {
        CompileConfig {
            commands: value.commands.into_iter().map(Into::into).collect(),
            dependencies: value.dependencies.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RunConfig {
    pub command: ExecutableEntry,
    pub dependencies: Vec<Dependency>,
    pub health_check: Vec<HealthCheck>
}
impl From<ConfigScriptedRunConfig> for RunConfig {
    fn from(value: ConfigScriptedRunConfig) -> Self {
        RunConfig {
            command: value.command.into(),
            dependencies: value.dependencies.into_iter().map(Into::into).collect(),
            health_check: value.health_check.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum HealthCheck {
    /// A health check in the form of a HTTP response, made to the given URL with the given HTTP method. The check is
    /// considered OK if something responds to the call within [timeout_millis] milliseconds with a status of [status]
    Http {
        url: String,
        method: String,
        timeout_millis: u64,
        status: u16
    },
    /// A health check in the form of an open port. The check is considered OK if the given [port] is is listening in
    /// the OS.
    Port {
        port: u16
    }
}
impl From<ConfigHealthCheck> for HealthCheck {
    fn from(value: ConfigHealthCheck) -> Self {
        match value {
            ConfigHealthCheck::Http { url, method, timeout_millis, status } => HealthCheck::Http {
                url, method, timeout_millis, status
            },
            ConfigHealthCheck::Port { port } => HealthCheck::Port { port },
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
impl Display for ExecutableEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.executable)?;
        for arg in &self.args {
            f.write_str(" ")?;
            write_escaped_str!(f, arg);
        }

        if !self.env.is_empty() {
            f.write_str(" (env: ")?;
            // Place env keys into a tmp variable so that we can sort them
            let mut env_keys: Vec<String> = self.env.keys().map(Clone::clone).collect();
            env_keys.sort();
            for key in env_keys {
                write_escaped_str!(f, &key);
                f.write_str("=")?;
                write_escaped_str!(f, self.env.get(&key).unwrap());
            }
            f.write_str(")")?;
        }

        Ok(())
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
    pub action: ServiceAction,
    pub should_run: bool,
    pub compile_status: CompileStatus,
    pub run_status: RunStatus,
    pub show_output: bool,
    pub auto_recompile: bool,
}
impl ServiceStatus {
    pub fn from(_profile: &Profile, _service: &Service) -> ServiceStatus {
        ServiceStatus {
            should_run: true,
            action: Recompile,
            auto_recompile: true,
            compile_status: CompileStatus::None,
            run_status: RunStatus::Stopped,
            show_output: true,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum ServiceAction {
    None,
    Recompile,
    Restart,
    Stop
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CompileStatus {
    None,
    Compiling(usize),
    PartiallyCompiled(usize),
    FullyCompiled,
    Failed
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RunStatus {
    Stopped,
    Running,
    Healthy,
    Failed
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OutputStore {
    pub outputs: HashMap<OutputKey, VecDeque<OutputLine>>,
    current_idx: u128
}
impl OutputStore {
    pub fn new() -> Self {
        OutputStore {
            outputs: HashMap::new(),
            current_idx: 1
        }
    }

    pub fn add_output(&mut self, key: &OutputKey, line: String) -> &OutputLine {
        if !self.outputs.contains_key(key) {
            self.outputs.insert(key.clone(), VecDeque::new());
        }
        let deque = self.outputs.get_mut(key).unwrap();
        deque.push_back(OutputLine {
            value: line,
            index: self.current_idx
        });
        self.current_idx += 1;
        // TODO move to a config field
        if deque.len() > 8096 {
            deque.pop_front();
        }

        deque.iter().last().unwrap()
    }

    pub fn query_lines(&self, num_lines: usize, max_idx: Option<u128>) -> Vec<(&OutputKey, &str)> {
        let max_idx = max_idx.unwrap_or(self.current_idx);
        let mut result: Vec<(&OutputKey, &str)> = Vec::with_capacity(num_lines);
        let mut bucket_indices: HashMap<OutputKey, Option<usize>> = self.outputs.iter()
            .map(|(key, lines)| {
                if lines.len() == 0 {
                    // If the bucket has 0 lines, then there's nothing we could ever return
                    (key.clone(), None)
                } else if lines.iter().last().map(|OutputLine { index, ..}| index <= &max_idx).unwrap_or(false) {
                    // If all lines have an index lower than the given max index, then the starting index is the length
                    // of the bucket
                    (key.clone(), (lines.len() - 1).into())
                } else {
                    // Otherwise find the partition point for elements at most the given index, and select the last
                    // index of the first partition
                    (
                        key.clone(),
                        (lines.partition_point(|OutputLine { index, .. }| {
                            index <= &max_idx
                        }) - 1).into()
                    )
                }
            }).collect();

        // Loop until the result vec is fully populated, or we run out of lines.
        while result.len() < num_lines && bucket_indices.iter().any(|(_, value)| value.is_some()) {
            // Figure out the bucket with the next line
            let next_bucket = self.outputs.iter().max_by_key(|(key, lines)| {
                if let Some(cur_idx) = bucket_indices.get(key).unwrap() {
                    lines.get(*cur_idx).unwrap().index
                } else {
                    0
                }
            }).unwrap().0;
            let cur_idx = bucket_indices.get(next_bucket).unwrap().unwrap();

            // Push the relevant line into the returned results
            result.push((next_bucket, &self.outputs.get(next_bucket).unwrap().get(cur_idx).unwrap().value));
            // Update the current index for the bucket
            if cur_idx > 0 {
                *bucket_indices.get_mut(next_bucket).unwrap() = Some(cur_idx - 1);
            } else {
                *bucket_indices.get_mut(next_bucket).unwrap() = None;
            }
        }

        result.into_iter().rev().collect()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct OutputKey {
    pub name: String,
    pub service_ref: String,
    pub kind: OutputKind
}
impl OutputKey {
    pub const STD: &'static str = "std";
    pub const CTRL: &'static str = "ctrl";

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
    pub index: u128
}