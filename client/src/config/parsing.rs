use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::{read_to_string, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use Vec;

use log::{debug, info};
use serde::Deserialize;
use walkdir::WalkDir;

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

    let settings: Settings = read_file(Path::new(dir).join("settings"))?;
    let mut services: Vec<ServiceDefinition> = Vec::new();
    let mut profiles: Vec<ProfileDefinition> = Vec::new();

    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let extension = path.extension().and_then(|ext| ext.to_str().to_owned()).unwrap_or_default();
        let stem = path.file_stem().and_then(|stem| stem.to_str().to_owned()).unwrap_or_default();

        match extension {
            "toml" | "yml" | "yaml" => {
                debug!("Checking path {path:?} as a potential configuration file")
            },
            _ => {
                debug!("Skipping path {path:?} due to invalid extension")
            },
        };

        if stem.ends_with(".service") {
            info!("Reading service configuration file {path:?}");
            services.push(read_file(path)?);
        } else if stem.ends_with(".profile") {
            info!("Reading profielervice configuration file {path:?}");
            profiles.push(read_file(path)?);
        }
    }

    Ok(Config {
        settings,
        conf_dir: dir.into(),
        services,
        profiles,
    })
}

fn read_file<'a, T : Deserialize<'a>, P: AsRef<Path>,>(path: P) -> Result<T, ConfigParsingError> {
    let extensions = ["toml", "yml", "yaml"];
    let mut path_with_ext: PathBuf = path.as_ref().to_path_buf();

    for ext in &extensions {
        path_with_ext.set_extension(ext);

        if path_with_ext.exists() {
            let result = match *ext {
                "toml" => read_toml::<T>(&path_with_ext),
                "yml" | "yaml" => read_yaml::<T>(&path_with_ext),
                _ => panic!("Unrecognized file extension: {ext}"),
            };

            if result.is_ok() {
                return result;
            }
        }
    }

    Err(ConfigParsingError {
        filename: path.as_ref().to_str().unwrap().to_string(),
        user_message: "No file with valid extension called {path_without_extension} found".to_owned(),
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

fn read_yaml<'a, T : Deserialize<'a>>(path: &Path) -> Result<T, ConfigParsingError> {
    let file = match File::open(path) {
        Ok(value) => Ok(value),
        Err(_) => {
            let error_path = path.to_str().unwrap().to_string();
            Err(ConfigParsingError {
                filename: error_path.clone(),
                user_message: format!("Error in opening file {error_path}")
            })
        }
    }?;
    let reader = BufReader::new(file);

    let deserializer = serde_yaml::Deserializer::from_reader(reader);
    let result = serde_path_to_error::deserialize(deserializer);

    match result {
        Ok(value) => Ok(value),
        Err(error) => {
            let error_path = error.path().to_string();
            let message = format!("{}", error.inner());

            Err(ConfigParsingError {
                filename: path.to_str().unwrap().to_string(),
                user_message: format!("Error in parsing at path {error_path}: {message}"),
            })
        }
    }
}
