use ratatui::layout::Size;
use crate::component::{Component, MeasurableComponent};
use crate::{FrameContext, UIResult};

pub struct FixedMeasurement<S, O, C>
where
    S: Default + 'static,
    C: Component<State = S, Output = O>,
{
    component: C,
    size: Size,
}

impl<S, O, C> Component for FixedMeasurement<S, O, C>
where
    S: Default + 'static,
    C: Component<State = S, Output = O>,
{

    type State = S;
    type Output = O;

    fn render(&self, context: &FrameContext, state: &mut Self::State) -> UIResult<Self::Output> {
        self.component.render(context, state)
    }
}

impl<S, O, C> MeasurableComponent for FixedMeasurement<S, O, C>
where
    S: Default + 'static,
    C: Component<State = S, Output = O>,
{
    fn measure(&self, context: &FrameContext, state: &Self::State) -> UIResult<Size> {
        Ok(self.size)
    }
}

pub trait WithMeasurement {
    type State : Default + 'static;
    type Output;
    type Component : Component<State = Self::State, Output = Self::Output>;

    fn with_measurement<
        X: Into<u16>,
        Y: Into<u16>
    >(self, width: X, height: Y) -> FixedMeasurement<Self::State, Self::Output, Self::Component>;
}

impl<S, O, C> WithMeasurement for C
where
    S: Default + 'static,
    C: Component<State = S, Output = O> {
    type State = S;
    type Output = O;
    type Component = C;

    fn with_measurement<
        X: Into<u16>,
        Y: Into<u16>
    >(self, width: X, height: Y) -> FixedMeasurement<Self::State, Self::Output, Self::Component> {
        FixedMeasurement {
            component: self,
            size: Size { width: width.into(), height: height.into() },
        }
    }
}
