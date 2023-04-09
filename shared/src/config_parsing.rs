use std::collections::HashMap;
use std::error::Error;
use std::fs::read_to_string;
use std::path::Path;
use Vec;

use serde::{Deserialize, Serialize};
use serde_aux::field_attributes::bool_true;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub server: ServerConfig
}

#[derive(Deserialize, Debug, Clone)]
pub struct ServerConfig {
    pub port: u16,
    #[serde(default="bool_true")]
    pub daemon: bool,
    #[serde(default="default_server_executable")]
    pub executable: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Service {
    #[serde(rename = "compilable")]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExecutableEntry {
    pub executable: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub artifact: Vec<ArtifactEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArtifactEntry {
    pub path: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServiceType {
    Compilable
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Profile {
    pub name: String,
    #[serde(default, rename = "service")]
    pub services: Vec<ServiceRef>
}
impl Profile {
    pub fn includes(&self, service: &Service) -> bool {
        self.services.iter().any(|reference| reference.references(service))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceRef {
    pub name: String
}
impl ServiceRef {
    pub fn references(&self, service: &Service) -> bool {
        service.name() == &self.name
    }
}

pub fn read_main_config(path: &str) -> Result<Config, Box<dyn Error>> {
    Ok(toml::from_str(&read_to_string(path)?)?)
}

pub fn read_service(path: &Path) -> Result<Service, Box<dyn Error>> {
    Ok(toml::from_str(&read_to_string(path)?)?)
}

pub fn read_profile(path: &Path) -> Result<Profile, Box<dyn Error>> {
    Ok(toml::from_str(&read_to_string(path)?)?)
}

fn default_server_executable() -> String {
    return String::from("./server")
}
