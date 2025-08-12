use crate::component::{Align, Cell, Component, Dir, Flow, FlowableArgs, MeasurableComponent};
use crate::{FrameContext, RenderArgs, UIError, UIResult};
use ratatui::layout::Size;
use ratatui::style::Color;

pub struct List<'a, ElementState, ElementOutput, Item, Element, CreateElement>
where
    ElementState: Default + 'static,
    ElementOutput: 'static,
    Element: MeasurableComponent<State = ElementState, Output = ElementOutput> + 'static,
    CreateElement: Fn(&Item, usize) -> Element,
{
    pub items: &'a Vec<Item>,
    pub create_element: CreateElement,
    pub dir: Dir,
}
impl<'a, ElementState, ElementOutput, Item, Element, CreateElement>
    List<'a, ElementState, ElementOutput, Item, Element, CreateElement>
where
    ElementState: Default + 'static,
    ElementOutput: 'static,
    Element: MeasurableComponent<State = ElementState, Output = ElementOutput> + 'static,
    CreateElement: Fn(&Item, usize) -> Element,
{
    pub fn new(items: &'a Vec<Item>, create_element: CreateElement) -> Self {
        Self {
            items,
            create_element,
            dir: Dir::UpDown,
        }
    }

    fn create_flow(&self) -> UIResult<Flow> {
        let mut flow = Flow::new()
            .dir(self.dir)
            .bg(Color::Red);
        let create_element = &self.create_element;

        for (index, item) in self.items.iter().enumerate() {
            flow = flow.element(create_element(item, index), FlowableArgs { fill: false });
        }

        Ok(flow)
    }
}

impl<'a, ElementState, ElementOutput, Item, Element, CreateElement> Component
    for List<'a, ElementState, ElementOutput, Item, Element, CreateElement>
where
    ElementState: Default + 'static,
    ElementOutput: 'static,
    Element: MeasurableComponent<State = ElementState, Output = ElementOutput> + 'static,
    CreateElement: Fn(&Item, usize) -> Element,
{
    type State = ();
    type Output = ();

    fn render(&self, context: &FrameContext, _: &mut Self::State) -> UIResult<Self::Output> {
        context.render_component(RenderArgs::new(&self.create_flow()?).key("list"))
    }
}

impl<'a, ElementState, ElementOutput, Item, Element, CreateElement> MeasurableComponent
    for List<'a, ElementState, ElementOutput, Item, Element, CreateElement>
where
    ElementState: Default + 'static,
    ElementOutput: 'static,
    Element: MeasurableComponent<State = ElementState, Output = ElementOutput> + 'static,
    CreateElement: Fn(&Item, usize) -> Element,
{
    fn measure(&self, context: &FrameContext, state: &Self::State) -> UIResult<Size> {
        self.create_flow()?.measure(context, state)
    }
}
