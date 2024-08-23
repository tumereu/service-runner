use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::text::Span;
use tui::widgets::{Paragraph};
use tui::Frame;

use crate::ui::widgets::{Renderable, Size};

#[derive(Debug, Default)]
pub struct Text {
    pub text: String,
    pub fg: Option<Color>,
}
impl Text {
    pub fn render<B>(self, rect: Rect, frame: &mut Frame<B>)
    where
        B: Backend,
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
