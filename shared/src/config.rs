use std::collections::HashMap;
use std::error::Error;
use std::fs::read_to_string;
use std::io::{BufReader, Result as IOResult};
use std::path::Path;
use serde::Deserialize;
use serde_aux::field_attributes::bool_true;
use walkdir::WalkDir;
use Vec;

pub use crate::config_parsing::{read_main_config, Config as TomlConfig, ServerConfig};
use crate::config_parsing::{read_service, Service};

#[derive(Debug)]
pub struct Config {
    pub server: ServerConfig,
    pub conf_dir: String,
    pub services: Vec<Service>,
}

pub fn read_config(dir: &str) -> Result<Config, Box<dyn Error>> {
    let main_config =  read_main_config(&format!("{dir}/config.toml"))?;
    let mut services: Vec<Service> = Vec::new();

    for entry in WalkDir::new(dir).follow_links(true).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path().clone();
        println!("{path:?}");
        if path.ends_with(".service.toml") {
            println!("ends with");
            services.push(read_service(&path)?)
        } else {
            println!("not ends with");
        }
    }

    Ok(Config {
        server: main_config.server,
        conf_dir: dir.into(),
        services
    })
}

fn default_server_executable() -> String {
    return String::from("./server")
}