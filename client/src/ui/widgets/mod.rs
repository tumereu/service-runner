use std::cmp::min;
use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;

pub use flex::*;
pub use list::*;

mod flex;
mod list;

#[derive(Clone, Copy)]
pub struct Size {
    pub width: u16,
    pub height: u16
}
impl Size {
    pub fn intersect(&self, other: Size) -> Size {
        (
            min(self.width, other.width),
            min(self.height, other.height)
        ).into()
    }
}

impl<X : Into<u16>, Y : Into<u16>> From<(X, Y)> for Size {
    fn from(value: (X, Y)) -> Self {
        Size {
            width: value.0.into(),
            height: value.1.into()
        }
    }
}

pub fn render_root<B, R>(root: R, frame: &mut Frame<B>) where B : Backend, R: Into<Renderable> {
    root.into().render(frame.size(), frame);
}

pub enum Renderable {
    Flex(Flex),
    List(List)
}
impl Renderable {
    fn render<B>(self, rect: Rect, frame: &mut Frame<B>) where B: Backend {
        match self {
            Renderable::Flex(flex) => flex.render(rect, frame),
            Renderable::List(list)  => list.render(rect, frame)
        }
    }

    fn measure(&self) -> Size {
        match self {
            Renderable::Flex(flex) => flex.measure(),
            Renderable::List(list) => list.measure()
        }
    }
}