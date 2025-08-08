use serde_derive::Deserialize;
use tui::style::Color;

#[derive(Deserialize, Debug, Clone)]
pub struct Theme {
    #[serde(default = "default_name_colors")]
    pub service_colors: Vec<Color>,
    #[serde(default = "default_name_colors")]
    pub block_colors: Vec<Color>,
    #[serde(default = "default_name_colors")]
    pub task_colors: Vec<Color>,
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
