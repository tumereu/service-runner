use std::fmt;
use std::ops::Deref;
use ratatui::style::Color;
use serde::{de, Deserialize, Deserializer};
use serde::de::Visitor;

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
