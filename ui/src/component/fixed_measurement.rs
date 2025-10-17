use crate::component::{Component, MeasurableComponent, StatefulComponent};
use crate::{FrameContext, UIResult};
use ratatui::layout::Size;

pub struct FixedMeasurement<O, C>
where
    C: Component<Output = O>,
{
    component: C,
    size: Size,
}

pub struct FixedStatefulMeasurement<O, S, C>
where
    C: StatefulComponent<Output = O, State = S>,
{
    component: C,
    size: Size,
}

impl<O, C> Component for FixedMeasurement<O, C>
where
    C: Component<Output = O>,
{

    type Output = O;

    fn render(self, context: &mut FrameContext) -> UIResult<Self::Output> {
        self.component.render(context)
    }
}

impl<O, C> MeasurableComponent for FixedMeasurement<O, C>
where
    C: Component<Output = O>,
{
    fn measure(&self, context: &FrameContext) -> UIResult<Size> {
        Ok(self.size)
    }
}

pub trait WithMeasurement {
    type Output;
    type Component : Component<Output = Self::Output>;

    fn with_measurement<
        X: Into<u16>,
        Y: Into<u16>
    >(self, width: X, height: Y) -> FixedMeasurement<Self::Output, Self::Component>;
}

impl<O, C> WithMeasurement for C
where
    C: Component<Output = O> {
    type Output = O;
    type Component = C;

    fn with_measurement<
        X: Into<u16>,
        Y: Into<u16>
    >(self, width: X, height: Y) -> FixedMeasurement<Self::Output, Self::Component> {
        FixedMeasurement {
            component: self,
            size: Size { width: width.into(), height: height.into() },
        }
    }
}

pub trait WithZeroMeasurement<O, C> where C : Component<Output = O> {
    fn with_zero_measurement(self) -> FixedMeasurement<O, C>;
}
impl <O, C, W> WithZeroMeasurement<O, C> for W
where
    C: Component<Output = O>,
    W: WithMeasurement<Output = O, Component = C>
{
    fn with_zero_measurement(self) -> FixedMeasurement<O, C> {
        self.with_measurement(0u16, 0u16)
    }
}
