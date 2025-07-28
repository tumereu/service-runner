use serde_derive::Deserialize;
use crate::config::{ProfileDefinition, ServiceDefinition};

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    #[serde(default)]
    pub autolaunch_profile: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub conf_dir: String,
    pub settings: Settings,
    pub services: Vec<ServiceDefinition>,
    pub profiles: Vec<ProfileDefinition>,
}

