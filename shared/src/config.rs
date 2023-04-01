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
pub struct Service {
    #[serde(default)]
    pub depends_on: Vec<String>,
    pub compile: Option<String>,
    pub run: String
}

pub fn read_config<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn Error>> {
    let contents = read_to_string(path)?;
    Ok(toml::from_str(&contents)?)
}

fn default_server_executable() -> String {
    return String::from("./server")
}