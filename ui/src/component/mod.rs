mod text;
mod cell;
mod flow;
mod vlist;

pub use text::*;
pub use cell::*;
pub use flow::*;
pub use vlist::*;

use ratatui::layout::Size;
use crate::frame_ctx::FrameContext;

pub trait Component {
    type State : Default + 'static;
    type Output;

    fn render(&self, context: &FrameContext, state: &mut Self::State) -> Self::Output;
}

pub trait MeasurableComponent : Component {
    fn measure(&self, context: &FrameContext, state: &Self::State) -> Size;
}
