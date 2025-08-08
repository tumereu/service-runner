use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::ui::widgets::{Renderable, Size};

#[derive(Debug, Default)]
pub struct Text {
    pub text: String,
    pub fg: Option<Color>,
}
impl Text {
    pub fn render(self, rect: Rect, frame: &mut Frame)
    {
        let mut style = Style::default();
        if let Some(fg) = self.fg {
            style = style.fg(fg);
        } else {
            style = style.fg(Color::Reset);
        }

        frame.render_widget(Paragraph::new(Span::styled(self.text.clone(), style)), rect);
    }

    pub fn measure(&self) -> Size {
        (self.text.len() as u16, 1_u16).into()
    }
}

impl From<Text> for Renderable {
    fn from(value: Text) -> Self {
        Renderable::Text(value)
    }
}
