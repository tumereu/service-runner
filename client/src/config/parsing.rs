use std::collections::{HashMap, HashSet};
use Vec;
use itertools::Itertools;
use log::{debug, info};
use serde::Deserialize;
use std::error::Error;
use std::fmt::{Display, Formatter, format};
use std::fs::{File, read_to_string};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::models::{Config, ProfileDefinition, ServiceDefinition};
use crate::config::{BlockId, RawSettings, ServiceId, Settings};

#[derive(Debug)]
pub struct ConfigurationError {
    filename: Option<String>,
    msg: String,
}
impl Display for ConfigurationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = &self.msg;
        if let Some(filename) = &self.filename {
            write!(f, "{msg} (file={filename})")
        } else {
            write!(f, "{msg}")
        }
    }
}
impl Error for ConfigurationError {}

pub fn read_config(dir: &str) -> Result<Config, ConfigurationError> {
    info!("Reading configuration froms directory {dir}");

    let settings_file = find_first_config_file(Path::new(dir).join("settings"))?;
    let raw_settings: RawSettings = read_file(&settings_file)?;
    let settings: Settings = raw_settings
        .try_into()
        .map_err(|error_msg| ConfigurationError {
            filename: Some(
                settings_file
                    .to_str()
                    .map(|path| path.to_string())
                    .unwrap_or_default(),
            ),
            msg: error_msg,
        })?;
    let mut services: Vec<ServiceDefinition> = Vec::new();
    let mut profiles: Vec<ProfileDefinition> = Vec::new();

    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str().to_owned())
            .unwrap_or_default();
        let stem = path
            .file_stem()
            .and_then(|stem| stem.to_str().to_owned())
            .unwrap_or_default();

        match extension {
            "toml" | "yml" | "yaml" => {
                debug!("Checking path {path:?} as a potential configuration file")
            }
            _ => {
                debug!("Skipping path {path:?} due to invalid extension")
            }
        };

        if stem.ends_with(".service") {
            info!("Reading service configuration file {path:?}");
            services.push(
                validate_service_definition(read_file(path)?).map_err(|error| {
                    ConfigurationError {
                        filename: Some(path.to_str().unwrap().to_string()),
                        msg: error.msg,
                    }
                })?,
            );
        } else if stem.ends_with(".profile") {
            info!("Reading profielervice configuration file {path:?}");
            profiles.push(read_file(path)?);
        }
    }

    let duplicate_service_ids: Vec<String> = services
        .iter()
        .chunk_by(|service| &service.id)
        .into_iter()
        .filter_map(|(_, grouped)| {
            let collected: Vec<_> = grouped.collect();
            if collected.len() > 1 {
                Some(collected)
            } else {
                None
            }
        })
        .map(|group| group.iter().next().unwrap().id.inner().to_owned())
        .collect();

    if duplicate_service_ids.len() > 0 {
        Err(ConfigurationError {
            filename: None,
            msg: format!(
                "Found non-unique service ids: {}",
                duplicate_service_ids.join(", ")
            ),
        })
    } else {
        Ok(Config {
            settings,
            conf_dir: dir.into(),
            services,
            profiles,
        })
    }
}

fn validate_service_definition(
    service: ServiceDefinition,
) -> Result<ServiceDefinition, ConfigurationError> {
    if service.id.inner().len() > 23 {
        return Err(
            ConfigurationError {
                filename: None,
                msg: format!(
                    "Service id {service_id} is longer than 23 characters",
                    service_id = service.id.inner()
                ),
            }
        );
    }
    
    let mut used_block_ids = HashSet::<BlockId>::new();
    for block in service.blocks.iter() {
        if block.id.inner().len() > 23 {
            return Err(
                ConfigurationError {
                    filename: None,
                    msg: format!(
                        "Block id {block_id} is longer than 23 characters",
                        block_id = block.id.inner(),
                    ),
                }
            );
        } else if used_block_ids.contains(&block.id) {
            return Err(
                ConfigurationError {
                    filename: None,
                    msg: format!(
                        "Block ids must be unique, but '{block_id}' appears more than once",
                        block_id = block.id.inner()
                    ),
                }
            );
        }
        used_block_ids.insert(block.id.clone());
    }
    
    Ok(service)
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

    let filename = path
        .as_ref()
        .to_str()
        .map(|path| path.to_string())
        .unwrap_or_default();
    Err(ConfigurationError {
        filename: Some(filename.clone()),
        msg: format!(
            "No suitable file found with path {path} (extensions {exts})",
            path = filename,
            exts = extensions.iter().map(|ext| format!(".{}", ext)).join(", ")
        ),
    })
}

fn read_file<'a, T: Deserialize<'a>, P: AsRef<Path>>(path: P) -> Result<T, ConfigurationError> {
    let extension = path
        .as_ref()
        .extension()
        .and_then(|ext| ext.to_str().to_owned())
        .unwrap_or_default();

    match extension {
        "toml" => read_toml::<T>(path.as_ref()),
        "yml" | "yaml" => read_yaml::<T>(path.as_ref()),
        _ => panic!("Unrecognized file extension: {extension}"),
    }
}

fn read_toml<'a, T: Deserialize<'a>>(path: &Path) -> Result<T, ConfigurationError> {
    let file_content = match read_to_string(path) {
        Ok(value) => Ok(value),
        Err(_) => {
            let error_path = path.to_str().unwrap().to_string();
            Err(ConfigurationError {
                filename: Some(error_path.clone()),
                msg: format!("Error in reading path {error_path} as string"),
            })
        }
    }?;

    let result = serde_path_to_error::deserialize(toml::Deserializer::new(&file_content));

    match result {
        Ok(value) => Ok(value),
        Err(error) => {
            let error_path = error.path().to_string();
            let message = error.inner().message();

            Err(ConfigurationError {
                filename: Some(path.to_str().unwrap().to_string()),
                msg: format!("Error in parsing at path {error_path}: {message}"),
            })
        }
    }
}

fn read_yaml<'a, T: Deserialize<'a>>(path: &Path) -> Result<T, ConfigurationError> {
    let file = match File::open(path) {
        Ok(value) => Ok(value),
        Err(_) => {
            let error_path = path.to_str().unwrap().to_string();
            Err(ConfigurationError {
                filename: Some(error_path.clone()),
                msg: format!("Error in opening file {error_path}"),
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
                filename: Some(path.to_str().unwrap().to_string()),
                msg: format!("Error in parsing at path {error_path}: {message}"),
            })
        }
    }
}
