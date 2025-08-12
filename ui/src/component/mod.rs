mod text;
mod cell;
mod flow;
mod list;

pub use text::*;
pub use cell::*;
pub use flow::*;
pub use list::*;

use ratatui::layout::Size;
use crate::frame_ctx::FrameContext;
use crate::{UIError, UIResult};

pub trait Component {
    type State : Default + 'static;
    type Output;

    fn render(&self, context: &FrameContext, state: &mut Self::State) -> UIResult<Self::Output>;
}

pub trait MeasurableComponent : Component {
    fn measure(&self, context: &FrameContext, state: &Self::State) -> UIResult<Size>;
}