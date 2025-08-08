use crate::config::parsing::ConfigurationError;
use ratatui::style::Color;
use serde::de::{Error, SeqAccess, Unexpected, Visitor};
use serde::Deserializer;
use serde_derive::Deserialize;
use std::fmt;

pub struct ThemeConfig;

#[derive(Debug, Clone)]
pub struct Theme {
    pub service_colors: Vec<Color>,
    pub block_colors: Vec<Color>,
    pub task_colors: Vec<Color>,
}
impl Default for Theme {
    fn default() -> Self {
        Self {
            service_colors: default_name_colors(),
            block_colors: default_name_colors(),
            task_colors: default_name_colors(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct RawTheme {
    #[serde(default)]
    pub service_colors: Vec<String>,
    #[serde(default)]
    pub block_colors: Vec<String>,
    #[serde(default)]
    pub task_colors: Vec<String>,
}
impl TryInto<Theme> for RawTheme {
    type Error = String;

    fn try_into(self) -> Result<Theme, Self::Error> {
        let mut theme = Theme::default();

        let RawTheme {
            service_colors,
            block_colors,
            task_colors
        } = self;

        if service_colors.len() > 0 {
            theme.service_colors = Self::try_into_theme_colors(service_colors)?
        }
        if block_colors.len() > 0 {
            theme.block_colors = Self::try_into_theme_colors(block_colors)?
        }
        if task_colors.len() > 0 {
            theme.block_colors = Self::try_into_theme_colors(task_colors)?
        }

        Ok(theme)
    }
}
impl RawTheme {
    fn try_into_theme_colors(colors: Vec<String>) -> Result<Vec<Color>, String> {
        colors
            .into_iter()
            .map(|color| Self::hex_to_rgb(&color))
            .into_iter()
            .collect()
    }

    fn hex_to_rgb(hex: &str) -> Result<Color, String> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);

        if hex.len() != 6 {
            return Err("A color hex must be 6 characters long".to_string());
        }

        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| format!("Invalid red component in {hex}"))?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| format!("Invalid greeb component in {hex}"))?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| format!("Invalid blue component in {hex}"))?;

        Ok(Color::Rgb(r, g, b))
    }
}

trait TryIntoColors {

}

fn default_name_colors() -> Vec<Color> {
    vec![
        Color::Rgb(255, 0, 0),
        Color::Rgb(255, 165, 0),
        Color::Rgb(255, 255, 0),
        Color::Rgb(0, 255, 0),
        Color::Rgb(0, 255, 255),
        Color::Rgb(0, 120, 180),
        Color::Rgb(128, 0, 128),
        Color::Rgb(255, 0, 255),
        Color::Rgb(255, 192, 203),
        Color::Rgb(255, 215, 0),
        Color::Rgb(255, 69, 0),
        Color::Rgb(0, 128, 0),
        Color::Rgb(139, 0, 139),
    ]
}
