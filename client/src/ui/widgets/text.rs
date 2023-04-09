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
