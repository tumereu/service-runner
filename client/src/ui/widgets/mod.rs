use std::cmp::max;

use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::text::Text;

pub use flex::*;
pub use list::*;
pub use size::*;

mod size;
mod flex;
mod list;

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