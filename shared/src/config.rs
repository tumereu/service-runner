use std::error::Error;
use Vec;

use walkdir::WalkDir;

use crate::config_parsing::{read_main_config, read_profile, read_service};
pub use crate::config_parsing::{ServerConfig, Profile, Service};

#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub conf_dir: String,
    pub services: Vec<Service>,
    pub profiles: Vec<Profile>,
}

pub fn read_config(dir: &str) -> Result<Config, Box<dyn Error>> {
    let main_config = read_main_config(&format!("{dir}/config.toml"))?;
    let mut services: Vec<Service> = Vec::new();
    let mut profiles: Vec<Profile> = Vec::new();

    for entry in WalkDir::new(dir).follow_links(true).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path().clone();
        let filename = entry.file_name().to_str().unwrap_or("");

        if filename.ends_with(".service.toml") {
            services.push(read_service(&path)?)
        } else if filename.ends_with(".profile.toml") {
            profiles.push(read_profile(&path)?)
        }
    }

    Ok(Config {
        server: main_config.server,
        conf_dir: dir.into(),
        services,
        profiles
    })
}

fn default_server_executable() -> String {
    return String::from("./server")
}