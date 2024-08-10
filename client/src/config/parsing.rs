use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::read_to_string;
use std::path::Path;
use Vec;
use log::info;
use serde::Deserialize;
use walkdir::WalkDir;

use crate::config::models::{Config, ProfileDefinition, ServiceDefinition};
use crate::config::{ScriptedCompileConfig, Settings};

#[derive(Debug)]
pub struct ConfigParsingError {
    inner: Box<dyn Error>,
    filename: String,
}
impl ConfigParsingError {
    pub fn new(inner: Box<dyn Error>, filename: &str) -> ConfigParsingError {
        ConfigParsingError {
            inner,
            filename: filename.to_string(),
        }
    }
}
impl Display for ConfigParsingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let filename = &self.filename;
        write!(f, "failed to parse {filename}")
    }
}
impl Error for ConfigParsingError {}

pub fn read_config(dir: &str) -> Result<Config, ConfigParsingError> {
    info!("Reading configuration froms directory {dir}");

    let main_file = format!("{dir}/settings.toml");
    let settings = read_settings(&main_file).map_err(|err| ConfigParsingError::new(err, &main_file))?;
    let mut services: Vec<ServiceDefinition> = Vec::new();
    let mut profiles: Vec<ProfileDefinition> = Vec::new();

    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path().clone();
        let filename = entry.file_name().to_str().unwrap_or("");

        info!("Reading configuration file {filename}");

        if filename.ends_with(".service.toml") {
            services
                .push(read_service(&path).map_err(|err| ConfigParsingError::new(err, filename))?)
        } else if filename.ends_with(".profile.toml") {
            profiles
                .push(read_profile(&path).map_err(|err| ConfigParsingError::new(err, filename))?)
        }
    }

    Ok(Config {
        settings,
        conf_dir: dir.into(),
        services,
        profiles,
    })
}

pub fn read_settings(path: &str) -> Result<Settings, Box<dyn Error>> {
    Ok(
        serde_path_to_error::deserialize(
            toml::Deserializer::new(&read_to_string(path)?)
        )?
    )
}

pub fn read_service(path: &Path) -> Result<ServiceDefinition, Box<dyn Error>> {
    Ok(
        serde_path_to_error::deserialize(
            toml::Deserializer::new(&read_to_string(path)?)
        )?
    )
}

pub fn read_profile(path: &Path) -> Result<ProfileDefinition, Box<dyn Error>> {
    Ok(toml::from_str(&read_to_string(path)?)?)
}
