mod color;

use macros::PartialStruct;
use ratatui::prelude::Color;
use crate::config::theme::color::ColorWrapper;

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
