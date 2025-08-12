use itertools::Itertools;
use log::{debug, info};
use serde::Deserialize;
use std::error::Error;
use std::fmt::{format, Display, Formatter};
use std::fs::{read_to_string, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use Vec;

use crate::config::models::{Config, ProfileDefinition, ServiceDefinition};
use crate::config::{RawSettings, Settings};

#[derive(Debug)]
pub struct ConfigurationError {
    pub filename: String,
    pub user_message: String,
}
impl Display for ConfigurationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let filename = &self.filename;
        write!(f, "failed to parse {filename}")
    }
}
impl Error for ConfigurationError {}

pub fn read_config(dir: &str) -> Result<Config, ConfigurationError> {
    info!("Reading configuration froms directory {dir}");

    let settings_file = find_first_config_file(Path::new(dir).join("settings"))?;
    let raw_settings: RawSettings = read_file(&settings_file)?;
    let settings: Settings = raw_settings.try_into()
        .map_err(|error_msg| ConfigurationError {
            filename: settings_file.to_str().map(|path| path.to_string()).unwrap_or_default(),
            user_message: error_msg,
        })?;
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

fn find_first_config_file<P: AsRef<Path>>(path: P) -> Result<PathBuf, ConfigurationError> {
    let extensions = ["toml", "yml", "yaml"];
    let mut path_with_ext: PathBuf = path.as_ref().to_path_buf();
    for ext in extensions.iter() {
        path_with_ext.set_extension(*ext);

        if path_with_ext.exists() {
            return Ok(path_with_ext);
        }
    }

    let filename = path.as_ref().to_str().map(|path| path.to_string()).unwrap_or_default();
    Err(ConfigurationError {
        filename: filename.clone(),
        user_message: format!(
            "No suitable file found with path {path} (extensions {exts})",
            path = filename,
            exts = extensions.iter().map(|ext| format!(".{}", ext)).join(", ")
        ),
    })
}

fn read_file<'a, T : Deserialize<'a>, P: AsRef<Path>,>(path: P) -> Result<T, ConfigurationError> {
    let extension = path.as_ref().extension().and_then(|ext| ext.to_str().to_owned()).unwrap_or_default();

    match extension {
        "toml" => read_toml::<T>(path.as_ref()),
        "yml" | "yaml" => read_yaml::<T>(path.as_ref()),
        _ => panic!("Unrecognized file extension: {extension}"),
    }
}

fn read_toml<'a, T : Deserialize<'a>>(path: &Path) -> Result<T, ConfigurationError> {
    let file_content = match read_to_string(path) {
        Ok(value) => Ok(value),
        Err(_) => {
            let error_path = path.to_str().unwrap().to_string();
            Err(ConfigurationError {
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

            Err(ConfigurationError {
                filename: path.to_str().unwrap().to_string(),
                user_message: format!("Error in parsing at path {error_path}: {message}"),
            })
        }
    }
}

fn read_yaml<'a, T : Deserialize<'a>>(path: &Path) -> Result<T, ConfigurationError> {
    let file = match File::open(path) {
        Ok(value) => Ok(value),
        Err(_) => {
            let error_path = path.to_str().unwrap().to_string();
            Err(ConfigurationError {
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

            Err(ConfigurationError {
                filename: path.to_str().unwrap().to_string(),
                user_message: format!("Error in parsing at path {error_path}: {message}"),
            })
        }
    }
}
