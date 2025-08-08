use serde_derive::Deserialize;

use crate::config::{ProfileDefinition, ServiceDefinition};
use crate::config::models::theme::Theme;

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    #[serde(default)]
    pub autolaunch_profile: Option<String>,
    pub theme: Theme,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub conf_dir: String,
    pub settings: Settings,
    pub services: Vec<ServiceDefinition>,
    pub profiles: Vec<ProfileDefinition>,
}

