mod text;
mod cell;

pub use text::*;
pub use cell::*;

use crate::canvas::FrameContext;
use crate::space::Size;

pub trait Component {
    type State : Default + 'static;
    type Output;

    fn render(&self, context: &FrameContext, state: &mut Self::State) -> Self::Output;
}

pub trait MeasurableComponent : Component {
    fn measure(&self, context: &FrameContext, state: &Self::State) -> Option<Size>;
}
