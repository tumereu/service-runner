use serde_derive::Deserialize;

use crate::config::models::theme::{RawTheme, Theme};
use crate::config::{ProfileDefinition, ServiceDefinition};
use crate::config::models::keybinds::Keybinds;

#[derive(Debug, Clone)]
pub struct Settings {
    pub autolaunch_profile: Option<String>,
    pub theme: Theme,
    pub keybinds: Keybinds
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct RawSettings {
    pub autolaunch_profile: Option<String>,
    pub theme: RawTheme,
    pub keybinds: Keybinds,
}
impl TryInto<Settings> for RawSettings {
    type Error = String;

    fn try_into(self) -> Result<Settings, Self::Error> {
        Ok(Settings {
            autolaunch_profile: self.autolaunch_profile,
            theme: self.theme.try_into()?,
            keybinds: self.keybinds,
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

