use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;
use tui::text::Text as TuiText;
use tui::widgets::{Paragraph};
use crate::ui::widgets::{Renderable, Size};

pub struct Text {
    text: String
}
impl Text {
    pub fn new(text: String) -> Text {
        Text {
            text
        }
    }

    pub fn from(text: &str) -> Text {
        Text {
            text: text.to_string()
        }
    }

    pub fn ljust<U : Into<usize>>(self, length: U) -> Self {
        if length <= self.text.len() {
            self
        } else {
            let mut text = String::new();
            while text.len() < length.into() - self.text.len() {

            }
            text.push()
            String::from(" ").c
        }

        Text {
            text: format!("{: ^width$}", self.text, width = length.into().saturating_sub(self.text.len())),
            ..self
        }
    }

    pub fn render<B>(self, rect: Rect, frame: &mut Frame<B>) where B: Backend {
        frame.render_widget(
            Paragraph::new(TuiText::from(self.text.clone())),
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
