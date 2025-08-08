use serde_derive::Deserialize;

use crate::config::{ProfileDefinition, ServiceDefinition};
use crate::config::models::theme::{RawTheme, Theme};

#[derive(Debug, Clone)]
pub struct Settings {
    pub autolaunch_profile: Option<String>,
    pub theme: Theme,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RawSettings {
    #[serde(default)]
    pub autolaunch_profile: Option<String>,
    #[serde(default)]
    pub theme: RawTheme,
}
impl TryInto<Settings> for RawSettings {
    type Error = String;

    fn try_into(self) -> Result<Settings, Self::Error> {
        Ok(Settings {
            autolaunch_profile: self.autolaunch_profile,
            theme: self.theme.try_into()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub conf_dir: String,
    pub settings: Settings,
    pub services: Vec<ServiceDefinition>,
    pub profiles: Vec<ProfileDefinition>,
}

