use std::vec;
use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans, Text as TuiText};
use tui::widgets::{Paragraph};
use crate::ui::widgets::{Renderable, Size};

pub struct Text {
    text: String,
    fg: Option<Color>,
}
impl Text {
    pub fn new(text: String) -> Text {
        Text {
            text,
            fg: None
        }
    }

    pub fn from(text: &str) -> Text {
        Text {
            text: text.to_string(),
            fg: None
        }
    }

    pub fn fg(self, color: Option<Color>) -> Text {
        Text {
            fg: color,
            ..self
        }
    }


    pub fn render<B>(self, rect: Rect, frame: &mut Frame<B>) where B: Backend {
        let mut style = Style::default();
        if let Some(fg) = self.fg {
            style = style.fg(fg);
        }

        frame.render_widget(
            Paragraph::new(Span::styled(self.text.clone(), style)),
            rect
        );
    }

    pub fn measure(&self) -> Size {
        (self.text.len() as u16, 1 as u16).into()
    }
}

impl From<Text> for Renderable {
    fn from(value: Text) -> Self {
        Renderable::Text(value)
    }
}
