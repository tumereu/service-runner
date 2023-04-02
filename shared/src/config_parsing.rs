use std::collections::HashMap;
use std::error::Error;
use std::fs::read_to_string;
use std::io::{BufReader, Result as IOResult};
use std::path::Path;
use serde::Deserialize;
use serde_aux::field_attributes::bool_true;
use Vec;

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

#[derive(Deserialize, Debug, Clone)]
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
            Service::Compilable { name, dir: _, compile: _, run: _, reset: _ } => &name,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ExecutableEntry {
    pub executable: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub artifact: Vec<ArtifactEntry>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ArtifactEntry {
    pub path: String,
    pub name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub enum ServiceType {
    Compilable
}

#[derive(Deserialize, Debug, Clone)]
pub struct Profile {
    pub name: String,
    #[serde(default, rename = "service")]
    pub services: Vec<ServiceRef>
}

#[derive(Deserialize, Debug, Clone)]
pub struct ServiceRef {
    pub name: String
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
