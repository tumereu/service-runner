use std::error::Error;
use std::fs::read_to_string;
use std::path::Path;
use Vec;

use serde::Deserialize;
use walkdir::WalkDir;

use crate::config::models::{Config, Profile, ServerConfig, Service};

#[derive(Deserialize, Debug)]
pub struct MainConfig {
    pub server: ServerConfig
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


pub fn read_main_config(path: &str) -> Result<MainConfig, Box<dyn Error>> {
    Ok(toml::from_str(&read_to_string(path)?)?)
}

pub fn read_service(path: &Path) -> Result<Service, Box<dyn Error>> {
    Ok(toml::from_str(&read_to_string(path)?)?)
}

pub fn read_profile(path: &Path) -> Result<Profile, Box<dyn Error>> {
    Ok(toml::from_str(&read_to_string(path)?)?)
}
