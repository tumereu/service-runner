use std::cmp::{max, min};
use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::Block;
use crate::ui::widgets::{Renderable, Size};

pub struct Container {
    bg: Option<Color>,
    padding_left: u16,
    padding_right: u16,
    padding_top: u16,
    padding_bottom: u16,
    min_width: Option<u16>,
    min_height: Option<u16>,
    align: Align,
    child: Box<Renderable>
}
impl Container {
    pub fn render<B>(self, rect: Rect, frame: &mut Frame<B>) where B: Backend {
        if let Some(bg) = self.bg {
            frame.render_widget(
                Block::default().style(Style::default().bg(bg)),
                rect
            );
        }

        let size = self.measure();

        // TODO padding is not correct in all use cases
        let x = match self.align {
            Align::TopLeft | Align::CenterLeft | Align::BottomLeft => {
                rect.x + self.padding_left
            },
            Align::TopCenter | Align::Center | Align::BottomCenter => {
                rect.x + (rect.width - size.width) / 2
            },
            Align::TopRight | Align::CenterRight | Align::BottomRight => {
                rect.x + rect.width - size.width - self.padding_right
            }
        };

        let y = match self.align {
            Align::TopLeft | Align::TopCenter | Align::TopRight => {
                rect.y + self.padding_top
            },
            Align::CenterLeft | Align::Center | Align::CenterRight => {
                rect.y + (rect.height - size.height) / 2
            },
            Align::BottomLeft | Align::BottomCenter | Align::BottomRight => {
                rect.y + rect.height - size.height - self.padding_bottom
            }
        };

        let child_rect = Rect {
            x,
            y,
            width: size.width - self.padding_left - self.padding_right,
            height: size.height - self.padding_top - self.padding_bottom
        };

        self.child.render(child_rect, frame);
    }

    pub fn measure(&self) -> Size {
        let child_rect = self.child.measure();

        let mut width = child_rect.width + self.padding_left + self.padding_right;
        if let Some(min_width) = self.min_width {
            width = max(width, min_width)
        }

        let mut height = child_rect.height + self.padding_top + self.padding_bottom;
        if let Some(min_height) = self.min_height {
            height = max(height, min_height)
        }

        (width, height).into()
    }

    pub fn from<R : Into<Renderable>>(child: R) -> Container {
        Container {
            child: Box::new(child.into()),
            bg: None,
            padding_left: 0,
            padding_right: 0,
            padding_top: 0,
            padding_bottom: 0,
            min_height: None,
            min_width: None,
            align: Align::TopLeft
        }
    }

    pub fn bg(self, color: Color) -> Container {
        Container {
            bg: Some(color),
            ..self
        }
    }

    pub fn pad_left(self, padding: u16) -> Container {
        Container {
            padding_left: padding,
            ..self
        }
    }

    pub fn pad_right(self, padding: u16) -> Container {
        Container {
            padding_right: padding,
            ..self
        }
    }

    pub fn pad_top(self, padding: u16) -> Container {
        Container {
            padding_top: padding,
            ..self
        }
    }

    pub fn pad_bottom(self, padding: u16) -> Container {
        Container {
            padding_bottom: padding,
            ..self
        }
    }

    pub fn align(self, align: Align) -> Container {
        Container {
            align,
            ..self
        }
    }

    pub fn min_width(self, min_width: u16) -> Container {
        Container {
            min_width: Some(min_width),
            ..self
        }
    }

    pub fn min_height(self, min_height: u16) -> Container {
        Container {
            min_height: Some(min_height),
            ..self
        }
    }
}

pub enum Align {
    TopLeft, TopCenter, TopRight,
    CenterLeft, Center, CenterRight,
    BottomLeft, BottomCenter, BottomRight
}

impl From<Container> for Renderable {
    fn from(value: Container) -> Self {
        Renderable::Container(value)
    }
}
