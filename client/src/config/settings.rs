use serde_derive::Deserialize;

use crate::config::keybinds::Keybinds;
use crate::config::theme::Theme;
use crate::config::{PartialTheme, ProfileDefinition, ServiceDefinition};
use crate::config::keybinds::PartialKeybinds;

#[derive(Debug, Clone, Default)]
pub struct Settings {
    pub autolaunch_profile: Option<String>,
    pub theme: Theme,
    pub keybinds: Keybinds,
    pub log_file: Option<String>,
}
impl From<Vec<PartialSettings>> for Settings {
    fn from(mut value: Vec<PartialSettings>) -> Self {
        let mut settings = Settings::default();

        value.sort_by_key(|partial_settings| partial_settings.load_order);
        for partial in value {
            partial.apply_to(&mut settings);
        }

        settings
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct PartialSettings {
    pub load_order: i32,
    #[serde(default)]
    pub autolaunch_profile: Option<String>,
    /// File path for writing the logs of the runner application itself.
    #[serde(default)]
    pub log_file: Option<String>,
    #[serde(default)]
    pub theme: PartialTheme,
    #[serde(default)]
    pub keybinds: PartialKeybinds,
}
impl PartialSettings {
    pub fn apply_to(self, settings: &mut Settings) {
        self.theme.apply_to(&mut settings.theme);
        self.keybinds.apply_to(&mut settings.keybinds);
        if let Some(autolaunch_profile) = self.autolaunch_profile {
            settings.autolaunch_profile = Some(autolaunch_profile);
        }
        if let Some(log_file) = self.log_file {
            settings.log_file = Some(log_file);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub conf_dir: String,
    pub settings: Settings,
    pub services: Vec<ServiceDefinition>,
    pub profiles: Vec<ProfileDefinition>,
}

