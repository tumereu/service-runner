use std::cmp::min;

use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;

pub use flow::*;
pub use list::*;
pub use text::*;
pub use spinner::*;
pub use cell::*;

mod flow;
mod list;
mod text;
mod spinner;
mod cell;

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

    pub fn empty() -> Size {
        Size {
            width: 0,
            height: 0
        }
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

#[derive(Debug)]
pub enum Renderable {
    Flow(Flow),
    Cell(Cell),
    List(List),
    Text(Text),
    Spinner(Spinner),
}
impl Renderable {
    fn render<B>(self, rect: Rect, frame: &mut Frame<B>) where B: Backend {
        match self {
            Renderable::Flow(flow) => flow.render(rect, frame),
            Renderable::Cell(cell) => cell.render(rect, frame),
            Renderable::List(list) => list.render(rect, frame),
            Renderable::Text(text) => text.render(rect, frame),
            Renderable::Spinner(spinner) => spinner.render(rect, frame),
        }
    }

    fn measure(&self) -> Size {
        match self {
            Renderable::Flow(flow) => flow.measure(),
            Renderable::Cell(cell) => cell.measure(),
            Renderable::List(list) => list.measure(),
            Renderable::Text(text) => text.measure(),
            Renderable::Spinner(spinner) => spinner.measure(),
        }
    }
}