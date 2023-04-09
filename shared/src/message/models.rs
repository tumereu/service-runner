use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::config::{
    Profile as ConfigProfile,
    Service as ConfigService,
    ExecutableEntry as ConfigExecutableEntry,
    ArtifactEntry as ConfigArtifactEntry
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
    pub artifacts: Vec<ArtifactEntry>,
}
impl From<ConfigExecutableEntry> for ExecutableEntry {
    fn from(value: ConfigExecutableEntry) -> Self {
        ExecutableEntry {
            executable: value.executable,
            args: value.args,
            env: value.env,
            artifacts: value.artifacts.into_iter().map(|artifact| artifact.into()).collect()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArtifactEntry {
    pub path: String,
    pub name: String,
}
impl From<ConfigArtifactEntry> for ArtifactEntry {
    fn from(value: ConfigArtifactEntry) -> Self {
        ArtifactEntry {
            path: value.path,
            name: value.name
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