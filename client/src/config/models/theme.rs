use ratatui::style::Color;
use serde::de::{Error, SeqAccess, Visitor};
use serde::Deserializer;
use serde_derive::Deserialize;

#[derive(Debug, Clone)]
pub struct Theme {
    pub service_colors: Vec<Color>,
    pub source_colors: Vec<Color>,
    pub active_color: Color,
    pub waiting_to_process_color: Color,
    pub processing_color: Color,
    pub error_color: Color,
    pub inactive_color: Color,
    pub idle_color: Color,
    pub focused_element: Color,
    pub unfocused_element: Color,
}
impl Default for Theme {
    fn default() -> Self {
        Self {
            service_colors: default_name_colors(),
            source_colors: default_name_colors(),
            active_color: Color::Rgb(0, 140, 0),
            waiting_to_process_color: Color::Rgb(230, 127, 0),
            processing_color: Color::Rgb(230, 180, 0),
            error_color: Color::Rgb(180, 0, 0),
            inactive_color: Color::Gray,
            idle_color: Color::White,
            focused_element: Color::Rgb(180, 180, 0),
            unfocused_element: Color::Rgb(100, 100, 0),
        }
    }
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct RawTheme {
    #[serde(default)]
    pub service_colors: Vec<String>,
    #[serde(default)]
    pub source_colors: Vec<String>,
    #[serde(default)]
    pub active_color: Option<String>,
    #[serde(default)]
    pub waiting_to_process_color: Option<String>,
    #[serde(default)]
    pub processing_color: Option<String>,
    #[serde(default)]
    pub error_color: Option<String>,
    #[serde(default)]
    pub inactive_color: Option<String>,
    #[serde(default)]
    pub idle_color: Option<String>,
    #[serde(default)]
    pub focused_element: Option<String>,
    #[serde(default)]
    pub unfocused_element: Option<String>,
}
impl TryInto<Theme> for RawTheme {
    type Error = String;

    fn try_into(self) -> Result<Theme, Self::Error> {
        let mut theme = Theme::default();

        let RawTheme {
            service_colors,
            source_colors,
            active_color,
            waiting_to_process_color,
            processing_color,
            error_color,
            inactive_color,
            idle_color,
            focused_element,
            unfocused_element,
        } = self;

        if service_colors.len() > 0 {
            theme.service_colors = Self::try_into_theme_colors(service_colors, "service_colors")?
        }
        if source_colors.len() > 0 {
            theme.source_colors = Self::try_into_theme_colors(source_colors, "source_colors")?
        }

        theme.active_color = active_color
            .map(|color| Self::try_into_theme_color(color, "active_color"))
            .unwrap_or(Ok(theme.active_color))?;
        theme.waiting_to_process_color = waiting_to_process_color
            .map(|color| Self::try_into_theme_color(color, "waiting_to_process_color"))
            .unwrap_or(Ok(theme.waiting_to_process_color))?;
        theme.processing_color = processing_color
            .map(|color| Self::try_into_theme_color(color, "processing_color"))
            .unwrap_or(Ok(theme.processing_color))?;
        theme.error_color = error_color
            .map(|color| Self::try_into_theme_color(color, "error_color"))
            .unwrap_or(Ok(theme.error_color))?;
        theme.inactive_color = inactive_color
            .map(|color| Self::try_into_theme_color(color, "inactive_color"))
            .unwrap_or(Ok(theme.inactive_color))?;
        theme.idle_color = idle_color
            .map(|color| Self::try_into_theme_color(color, "idle_color"))
            .unwrap_or(Ok(theme.idle_color))?;

        theme.focused_element = focused_element
            .map(|color| Self::try_into_theme_color(color, "focused_element"))
            .unwrap_or(Ok(theme.focused_element))?;
        theme.unfocused_element = unfocused_element
            .map(|color| Self::try_into_theme_color(color, "unfocused_element"))
            .unwrap_or(Ok(theme.unfocused_element))?;

        Ok(theme)
    }
}
impl RawTheme {
    fn try_into_theme_color(color: String, property_name: &str) -> Result<Color, String> {
        Self::hex_to_rgb(&color).map_err(|err| format!("Error in {property_name}: {err}"))
    }

    fn try_into_theme_colors(
        colors: Vec<String>,
        property_name: &str,
    ) -> Result<Vec<Color>, String> {
        colors
            .into_iter()
            .map(|color| Self::hex_to_rgb(&color))
            .into_iter()
            .collect::<Result<Vec<Color>, String>>()
            .map_err(|err| format!("Error in {property_name}: {err}"))
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

trait TryIntoColors {}

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
