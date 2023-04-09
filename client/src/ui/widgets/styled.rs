use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::Block;
use crate::ui::widgets::{Renderable, Size};

pub struct Styled {
    bg: Option<Color>,
    padding_left: u16,
    padding_right: u16,
    padding_top: u16,
    padding_bottom: u16,
    child: Box<Renderable>
}
impl Styled {
    pub fn render<B>(self, rect: Rect, frame: &mut Frame<B>) where B: Backend {
        if let Some(bg) = self.bg {
            frame.render_widget(
                Block::default().style(Style::default().bg(bg)),
                rect
            );
        }

        let child_rect = Rect {
            x: rect.x + self.padding_left,
            y: rect.y + self.padding_top,
            width: rect.width - self.padding_left - self.padding_right,
            height: rect.height - self.padding_top - self.padding_top
        };

        self.child.render(child_rect, frame);
    }

    pub fn measure(&self) -> Size {
        let child_rect = self.child.measure();

        (
            child_rect.width + self.padding_left + self.padding_right,
            child_rect.height + self.padding_top + self.padding_bottom
        ).into()
    }

    pub fn from(child: Renderable) -> Styled {
        Styled {
            child: Box::new(child),
            bg: None,
            padding_left: 0,
            padding_right: 0,
            padding_top: 0,
            padding_bottom: 0
        }
    }

    pub fn bg(self, color: Color) -> Styled {
        Styled {
            bg: Some(color),
            ..self
        }
    }

    pub fn pad_left(self, padding: u16) -> Styled {
        Styled {
            padding_left: padding,
            ..self
        }
    }

    pub fn pad_right(self, padding: u16) -> Styled {
        Styled {
            padding_right: padding,
            ..self
        }
    }

    pub fn pad_top(self, padding: u16) -> Styled {
        Styled {
            padding_top: padding,
            ..self
        }
    }

    pub fn pad_bottom(self, padding: u16) -> Styled {
        Styled {
            padding_bottom: padding,
            ..self
        }
    }
}

pub trait Styleable {
    fn styling(self) -> Styled;
}

impl<R : Into<Renderable>> Styleable for R {
    fn styling(self) -> Styled {
        Styled::from(self.into())
    }
}

impl From<Styled> for Renderable {
    fn from(value: Styled) -> Self {
        Renderable::Styled(value)
    }
}
