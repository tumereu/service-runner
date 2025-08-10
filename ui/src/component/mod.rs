mod text;

pub use text::*;

use crate::canvas::Canvas;
use crate::space::Size;
use crate::render_context::RenderContext;

pub trait Component {
    type State : Default + 'static;
    type Output;

    fn measure(&self, canvas: &Canvas, state: &mut Self::State) -> Measurement;
    fn render<'a>(&self, canvas: &Canvas, state: &'a mut Self::State) -> Self::Output;
    
}

#[derive(Debug, Clone, Default)]
pub struct Measurement {
    pub min: Option<Size>,
    pub max: Option<Size>,
}

