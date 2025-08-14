use crate::component::{ATTR_COLOR_HIGHLIGHT, ATTR_KEY_NAV_DOWN, ATTR_KEY_NAV_UP, Component, Dir, Flow, FlowableArgs, MeasurableComponent, StatefulComponent, ATTR_KEY_SELECT, List};
use crate::input::KeyMatcherQueryable;
use crate::{FrameContext, RenderArgs, UIError, UIResult};
use ratatui::layout::Size;
use ratatui::prelude::Style;
use ratatui::style::Color;
use ratatui::widgets::Block;

pub struct SimpleList<'a, ElementOutput, Item, Element, CreateElement>
where
    ElementOutput: 'static,
    Element: MeasurableComponent<Output = ElementOutput> + 'static,
    CreateElement: Fn(&Item, usize) -> UIResult<Element>,
{
    pub items: &'a Vec<Item>,
    pub create_element: CreateElement,
    pub id: String,
    pub hide_highlight: bool,
}
impl<'a, ElementOutput, Item, Element, CreateElement>
    SimpleList<'a, ElementOutput, Item, Element, CreateElement>
where
    ElementOutput: 'static,
    Element: MeasurableComponent<Output = ElementOutput> + 'static,
    CreateElement: Fn(&Item, usize) -> UIResult<Element>,
{
    pub fn new(id: &str, items: &'a Vec<Item>, create_element: CreateElement) -> Self {
        Self {
            id: id.to_string(),
            items,
            create_element,
            hide_highlight: false,
        }
    }

    pub fn highlight_visible(mut self, visible: bool) -> Self {
        self.hide_highlight = !visible;
        self
    }
}

#[derive(Default)]
pub struct ListState {
    pub scroll_offset: i32,
    pub selection: usize,
}

pub struct ListSelection {
    pub selected_index: usize,
}

impl<'a, ElementOutput, Item, Element, CreateElement> StatefulComponent
    for SimpleList<'a, ElementOutput, Item, Element, CreateElement>
where
    ElementOutput: 'static,
    Element: MeasurableComponent<Output = ElementOutput> + 'static,
    CreateElement: Fn(&Item, usize) -> UIResult<Element>,
{
    type State = ListState;
    type Output = Option<ListSelection>;

    fn state_id(&self) -> &str {
        &self.id
    }

    fn render(self, context: &mut FrameContext, state: &mut Self::State) -> UIResult<Self::Output> {
        struct ResolvedElement<Element> {
            element: Element,
            size: Size,
            index: usize,
        }

        if self.items.is_empty() {
            return Err(UIError::InvalidProp {
                msg: "List must have at least one item".to_string(),
            });
        }

        let create_element = &self.create_element;
        let self_size = context.size();

        let mut resolved_elements: Vec<ResolvedElement<Element>> =
            Vec::with_capacity(self.items.len());
        for (index, item) in self.items.iter().enumerate() {
            let element = create_element(item, index)?;
            let size = context.measure_component(&element)?;

            resolved_elements.push(ResolvedElement {
                element,
                size,
                index,
            });
        }

        if context
            .signals()
            .is_key_pressed(context.req_attr(ATTR_KEY_NAV_DOWN)?)
        {
            state.selection = state.selection.saturating_add(1);
        }
        state.selection = state.selection.min(resolved_elements.len() - 1);

        if context
            .signals()
            .is_key_pressed(context.req_attr(ATTR_KEY_NAV_UP)?)
        {
            state.selection = state.selection.saturating_sub(1)
        }
        let selection_mde = context.signals().is_key_pressed(context.req_attr(ATTR_KEY_SELECT)?);
        
        context.render_component(
            RenderArgs::new(
                List::new(
                    &format!("{}-list", self.id),
                    self.items,
                    self.create_element
                ).selection(if self.hide_highlight {
                    None
                } else {
                    Some(state.selection)
                })
            )
        )?;
        
        if selection_mde {
            Ok(Some(ListSelection {
                selected_index: state.selection,
            }))
        } else {
            Ok(None)
        }
    }
}

impl<'a, ElementOutput, Item, Element, CreateElement> MeasurableComponent
    for SimpleList<'a, ElementOutput, Item, Element, CreateElement>
where
    ElementOutput: 'static,
    Element: MeasurableComponent<Output = ElementOutput> + 'static,
    CreateElement: Fn(&Item, usize) -> UIResult<Element>,
{
    fn measure(&self, context: &FrameContext) -> UIResult<Size> {
        let create_element = &self.create_element;
        let mut height = 0;
        let mut width = 0;

        for (index, item) in self.items.iter().enumerate() {
            let element = create_element(item, index)?;
            let size = context.measure_component(&element)?;
            height += size.height;
            width = size.width.max(width);
        }

        Ok(Size::new(width, height))
    }
}
