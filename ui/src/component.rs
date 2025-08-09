use crate::canvas::Canvas;
use crate::space::Size;
use crate::state_store::StoreAccessContext;

pub trait Component<S> where S : Default + 'static {
    fn measure(&self, canvas: &Canvas, state: StoreAccessContext<S>) -> Measurement;
    fn render(&self, canvas: &Canvas, state: StoreAccessContext<S>);
}

#[derive(Debug, Clone, Default)]
pub struct Measurement {
    pub min: Option<Size>,
    pub max: Option<Size>,
}

