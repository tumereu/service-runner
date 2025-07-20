use log::info;
use serde::Deserialize;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::read_to_string;
use std::path::Path;
use walkdir::WalkDir;
use Vec;

use crate::config::models::{Config, ProfileDefinition, ServiceDefinition};
use crate::config::Settings;

#[derive(Debug)]
pub struct ConfigParsingError {
    pub filename: String,
    pub user_message: String,
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
    let settings: Settings = read_toml(Path::new(&main_file))?;
    let mut services: Vec<ServiceDefinition> = Vec::new();
    let mut profiles: Vec<ProfileDefinition> = Vec::new();

    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let filename = entry.file_name().to_str().unwrap_or("");

        info!("Reading configuration file {filename}");

        if filename.ends_with(".service.toml") {
            services.push(read_toml(path)?);
        } else if filename.ends_with(".profile.toml") {
            profiles.push(read_toml(path)?)
        }
    }

    Ok(Config {
        settings,
        conf_dir: dir.into(),
        services,
        profiles,
    })
}

fn read_toml<'a, T : Deserialize<'a>>(path: &Path) -> Result<T, ConfigParsingError> {
    let file_content = match read_to_string(path) {
        Ok(value) => Ok(value),
        Err(_) => {
            let error_path = path.to_str().unwrap().to_string();
            Err(ConfigParsingError {
                filename: error_path.clone(),
                user_message: format!("Error in reading path {error_path} as string")
            })
        }
    }?;

    let result = serde_path_to_error::deserialize(
        toml::Deserializer::new(&file_content)
    );

    match result {
        Ok(value) => Ok(value),
        Err(error) => {
            let error_path = error.path().to_string();
            let message = error.inner().message();

            Err(ConfigParsingError {
                filename: path.to_str().unwrap().to_string(),
                user_message: format!("Error in parsing at path {error_path}: {message}"),
            })
        }
    }
}
