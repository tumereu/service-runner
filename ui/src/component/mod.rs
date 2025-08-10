mod text;

pub use text::*;

use crate::canvas::FrameContext;
use crate::space::Size;

pub trait Component {
    type State : Default + 'static;
    type Output;

    fn measure(&self, context: &FrameContext, state: &Self::State) -> Measurement;
    fn render(&self, context: &FrameContext, state: &mut Self::State) -> Self::Output;

}

#[derive(Debug, Clone, Default)]
pub struct Measurement {
    pub min: Option<Size>,
    pub max: Option<Size>,
}

