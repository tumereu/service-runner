mod text;
mod cell;
mod flow;

use ratatui::layout::Size;
pub use text::*;
pub use cell::*;
pub use flow::*;

use crate::canvas::FrameContext;

pub trait Component {
    type State : Default + 'static;
    type Output;

    fn render(&self, context: &FrameContext, state: &mut Self::State) -> Self::Output;
}

pub trait MeasurableComponent : Component {
    fn measure(&self, context: &FrameContext, state: &Self::State) -> Size;
}
