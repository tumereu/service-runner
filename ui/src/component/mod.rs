mod cell;
mod flow;
mod list;
mod text;
mod fixed_measurement;

pub use cell::*;
pub use fixed_measurement::*;
pub use flow::*;
pub use list::*;
pub use text::*;

use crate::frame_ctx::FrameContext;
use crate::UIResult;
use ratatui::layout::Size;

pub trait Component {
    type State: Default + 'static;
    type Output;

    fn render(&self, context: &FrameContext, state: &mut Self::State) -> UIResult<Self::Output>;
}

pub trait MeasurableComponent: Component {
    fn measure(&self, context: &FrameContext, state: &Self::State) -> UIResult<Size>;
}
