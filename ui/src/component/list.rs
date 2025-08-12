use ratatui::layout::{Size};
use crate::component::{Cell, Component, Flow, FlowableArgs, MeasurableComponent};
use crate::{FrameContext, RenderArgs, RenderError, UIResult};

pub struct List<ElementState, ElementOutput, Item, Element, CreateElement>
where
    ElementState: Default + 'static,
    ElementOutput: 'static,
    Element: MeasurableComponent<State=ElementState, Output=ElementOutput> + 'static,
    CreateElement: Fn(&Item, usize) -> Element,
{
    pub items: Vec<Item>,
    pub create_element: Option<CreateElement>,
}
impl<ElementState, ElementOutput, Item, Element, CreateElement> List<ElementState, ElementOutput, Item, Element, CreateElement>
where
    ElementState: Default + 'static,
    ElementOutput: 'static,
    Element: MeasurableComponent<State=ElementState, Output=ElementOutput> + 'static,
    CreateElement: Fn(&Item, usize) -> Element,
{
    pub fn new(items: Vec<Item>) -> Self {
        Self {
            items,
            create_element: None,
        }
    }

    fn create_flow(&self) -> UIResult<Flow> {
        let mut flow = Flow::new();
        let create_element = self.create_element.as_ref().ok_or(RenderError::ComponentArg {
            msg: "Missing required prop 'create_element'".to_string()
        })?;

        for (index, item) in self.items.iter().enumerate() {
            let cell = Cell::new(create_element(item, index));
            flow = flow.element(cell, FlowableArgs { fill: false });
        }

        Ok(flow)
    }
}

impl<ElementState, ElementOutput, Item, Element, CreateElement> Component for List<ElementState, ElementOutput, Item, Element, CreateElement>
where
    ElementState: Default + 'static,
    ElementOutput: 'static,
    Element: MeasurableComponent<State=ElementState, Output=ElementOutput> + 'static,
    CreateElement: Fn(&Item, usize) -> Element,
{
    type State = ();
    type Output = ();

    fn render(&self, context: &FrameContext, _: &mut Self::State) -> UIResult<Self::Output> {
        let flow = self.create_flow()?;

        context.render_component(
            RenderArgs::new(&self.create_flow()?)
                .key("list")
        )
    }
}

impl<ElementState, ElementOutput, Item, Element, CreateElement> MeasurableComponent for List<ElementState, ElementOutput, Item, Element, CreateElement>
where
    ElementState: Default + 'static,
    ElementOutput: 'static,
    Element: MeasurableComponent<State=ElementState, Output=ElementOutput> + 'static,
    CreateElement: Fn(&Item, usize) -> Element,
{
    fn measure(&self, context: &FrameContext, state: &Self::State) -> UIResult<Size> {
        self.create_flow()?.measure(context, state)
    }
}
