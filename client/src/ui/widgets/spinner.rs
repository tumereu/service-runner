use std::time::Instant;

use once_cell::sync::Lazy;
use ratatui::backend::Backend;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::ui::widgets::{Renderable, Size};

static REFERENCE_INSTANT: Lazy<Instant> = Lazy::new(|| Instant::now());
const SPINNER_CHARS: &'static [&'static str] = &["⠋", "⠙", "⠸", "⠴", "⠦", "⠇"];

#[derive(Default, Debug)]
pub struct Spinner {
    pub active: bool,
    pub fg: Option<Color>,
}
impl Spinner {
    pub fn render(self, rect: Rect, frame: &mut Frame)
    {
        let mut style = Style::default();
        if let Some(fg) = self.fg {
            style = style.fg(fg);
        }

        let phase: u128 = (Instant::now()
            .duration_since(*REFERENCE_INSTANT)
            .as_millis()
            / 100)
            % (SPINNER_CHARS.len() as u128);

        let icon = if !self.active {
            " "
        } else {
            &SPINNER_CHARS[phase as usize]
        };

        frame.render_widget(Paragraph::new(Span::styled(icon, style)), rect);
    }

    pub fn measure(&self) -> Size {
        (1 as u16, 1 as u16).into()
    }
}

impl From<Spinner> for Renderable {
    fn from(value: Spinner) -> Self {
        Renderable::Spinner(value)
    }
}
