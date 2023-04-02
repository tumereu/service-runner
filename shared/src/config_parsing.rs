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

#[derive(Deserialize, Debug)]
pub struct ServerConfig {
    pub port: u16,
    #[serde(default="bool_true")]
    pub daemon: bool,
    #[serde(default="default_server_executable")]
    pub executable: String
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Service {
    Compilable(CompilableService),
}
impl Service {
    pub fn name(&self) -> &String {
        match self {
            Service::Compilable(service) => &service.name
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct CompilableService {
    pub name: String,
    pub dir: String,
    pub compile: Vec<ExecutableEntry>,
    pub run: Vec<ExecutableEntry>,
    pub reset: Vec<ExecutableEntry>,
}

#[derive(Deserialize, Debug)]
pub struct ExecutableEntry {
    pub executable: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub artifact: Vec<ArtifactEntry>,
}

#[derive(Deserialize, Debug)]
pub struct ArtifactEntry {
    pub path: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub enum ServiceType {
    Compilable
}

pub fn read_main_config(path: &str) -> Result<Config, Box<dyn Error>> {
    Ok(toml::from_str(&read_to_string(path)?)?)
}

pub fn read_service(path: &Path) -> Result<Service, Box<dyn Error>> {
    Ok(toml::from_str(&read_to_string(path)?)?)
}

fn default_server_executable() -> String {
    return String::from("./server")
}
