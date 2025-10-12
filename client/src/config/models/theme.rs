use std::fmt;
use std::ops::Deref;
use ratatui::style::Color;
use serde::{de, Deserialize, Deserializer};
use serde::de::Visitor;
use macros::PartialStruct;

#[derive(Debug, Clone, PartialStruct)]
pub struct Theme {
    pub service_colors: Vec<ColorWrapper>,
    pub source_colors: Vec<ColorWrapper>,
    pub active_color: ColorWrapper,
    pub partially_active_color: ColorWrapper,
    pub waiting_to_process_color: ColorWrapper,
    pub processing_color: ColorWrapper,
    pub error_color: ColorWrapper,
    pub inactive_color: ColorWrapper,
    pub idle_color: ColorWrapper,
    pub focused_element: ColorWrapper,
    pub unfocused_element: ColorWrapper,
}
impl Default for Theme {
    fn default() -> Self {
        Self {
            service_colors: default_name_colors(),
            source_colors: default_name_colors(),
            active_color: Color::Rgb(0, 140, 0).into(),
            partially_active_color: Color::Rgb(0, 140, 140).into(),
            waiting_to_process_color: Color::Rgb(230, 127, 0).into(),
            processing_color: Color::Rgb(230, 180, 0).into(),
            error_color: Color::Rgb(180, 0, 0).into(),
            inactive_color: Color::Gray.into(),
            idle_color: Color::White.into(),
            focused_element: Color::Rgb(180, 180, 0).into(),
            unfocused_element: Color::Rgb(100, 100, 0).into(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ColorWrapper(pub Color);
impl Deref for ColorWrapper {
    type Target = Color;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<Color> for ColorWrapper {
    fn from(color: Color) -> Self {
        Self(color)
    }
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

impl<'de> Deserialize<'de> for ColorWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ColorVisitor;

        impl<'de> Visitor<'de> for ColorVisitor {
            type Value = ColorWrapper;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a color hex string like \"#ffcc00\" or \"ffcc00\"")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                hex_to_rgb(value)
                    .map(ColorWrapper)
                    .map_err(|err| E::custom(format!("Invalid color: {err}")))
            }
        }

        deserializer.deserialize_str(ColorVisitor)
    }
}

fn default_name_colors() -> Vec<ColorWrapper> {
    vec![
        Color::Rgb(255, 0, 0).into(),
        Color::Rgb(255, 165, 0).into(),
        Color::Rgb(255, 255, 0).into(),
        Color::Rgb(0, 255, 0).into(),
        Color::Rgb(0, 255, 255).into(),
        Color::Rgb(0, 120, 180).into(),
        Color::Rgb(128, 0, 128).into(),
        Color::Rgb(255, 0, 255).into(),
        Color::Rgb(255, 192, 203).into(),
        Color::Rgb(255, 215, 0).into(),
        Color::Rgb(255, 69, 0).into(),
        Color::Rgb(0, 128, 0).into(),
        Color::Rgb(139, 0, 139).into(),
    ]
}
