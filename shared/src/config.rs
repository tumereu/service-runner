use std::collections::HashMap;
use std::error::Error;
use std::fs::read_to_string;
use std::io::{BufReader, Result as IOResult};
use std::path::Path;
use serde::Deserialize;
use serde_aux::field_attributes::bool_true;
use Vec;

pub use crate::config_parsing::{read_main_config, Config as TomlConfig, ServerConfig};

#[derive(Debug)]
pub struct Config {
    pub server: ServerConfig,
    pub conf_dir: String
}

pub fn read_config(dir: &str) -> Result<Config, Box<dyn Error>> {
    let main_config =  read_main_config(&format!("{dir}/config.toml"))?;

    Ok(Config {
        server: main_config.server,
        conf_dir: dir.into()
    })
}

fn default_server_executable() -> String {
    return String::from("./server")
}