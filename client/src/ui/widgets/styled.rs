use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::Block;
use crate::ui::widgets::{Renderable, Size};

pub struct Styled {
    color: Color,
    child: Box<Renderable>
}
impl Styled {
    pub fn render<B>(&self, rect: Rect, frame: &mut Frame<B>) where B: Backend {
        frame.render_widget(
            Block::default().style(Style::default().bg(self.color)),
            rect
        );
        self.child.render(rect, frame);
    }

    pub fn measure(&self) -> Size {
        self.child.measure()
    }
}

pub trait Styleable {
    fn bg(self, color: Color) -> Styled;
}

impl<R : Into<Renderable>> Styleable for R {
    fn bg(self, color: Color) -> Styled {
        Styled {
            color,
            child: Box::new(self.into())
        }
    }
}

impl From<Styled> for Renderable {
    fn from(value: Styled) -> Self {
        Renderable::Styled(value)
    }
}
