use std::cmp::min;

use ratatui::backend::Backend;
use ratatui::layout::Rect;
use ratatui::Frame;

pub use cell::*;
pub use flow::*;
pub use list::*;
pub use output_display::*;
pub use spinner::*;
pub use text::*;
pub use toggle::*;

mod cell;
mod flow;
mod list;
mod toggle;
mod spinner;
mod text;
mod output_display;

#[derive(Clone, Copy)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}
impl Size {
    pub fn intersect(&self, other: Size) -> Size {
        (min(self.width, other.width), min(self.height, other.height)).into()
    }

    pub fn empty() -> Size {
        Size {
            width: 0,
            height: 0,
        }
    }
}

impl<X: Into<u16>, Y: Into<u16>> From<(X, Y)> for Size {
    fn from(value: (X, Y)) -> Self {
        Size {
            width: value.0.into(),
            height: value.1.into(),
        }
    }
}

pub fn render_root<R>(root: R, frame: &mut Frame)
where
    R: Into<Renderable>,
{
    root.into().render(frame.size(), frame);
}

pub fn render_at_pos<R>(element: R, pos: (u16, u16), frame: &mut Frame)
    where
        R: Into<Renderable>,
{
    element.into().render_at_pos(pos, frame);
}

#[derive(Debug)]
pub enum Renderable {
    Flow(Flow),
    Cell(Cell),
    List(List),
    Toggle(Toggle),
    Text(Text),
    Spinner(Spinner),
    OutputDisplay(OutputDisplay),
}
impl Renderable {
    fn render(self, rect: Rect, frame: &mut Frame)
    {
        match self {
            Renderable::Flow(flow) => flow.render(rect, frame),
            Renderable::Cell(cell) => cell.render(rect, frame),
            Renderable::List(list) => list.render(rect, frame),
            Renderable::Toggle(toggle) => toggle.render(rect, frame),
            Renderable::Text(text) => text.render(rect, frame),
            Renderable::Spinner(spinner) => spinner.render(rect, frame),
            Renderable::OutputDisplay(display) => display.render(rect, frame),
        }
    }

    fn measure(&self) -> Size {
        match self {
            Renderable::Flow(flow) => flow.measure(),
            Renderable::Cell(cell) => cell.measure(),
            Renderable::List(list) => list.measure(),
            Renderable::Toggle(toggle) => toggle.measure(),
            Renderable::Text(text) => text.measure(),
            Renderable::Spinner(spinner) => spinner.measure(),
            Renderable::OutputDisplay(display) => display.measure(),
        }
    }

    pub fn render_at_pos(self, pos: (u16, u16), frame: &mut Frame)
    {
        let size = self.measure();
        let width = min(size.width, frame.size().width.saturating_sub(pos.0));
        let height = min(size.height, frame.size().height.saturating_sub(pos.1));

        self.render(
            Rect::new(pos.0, pos.1, width, height),
            frame
        );
    }
}
