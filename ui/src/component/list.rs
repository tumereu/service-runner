use crate::component::{
    ATTR_COLOR_HIGHLIGHT, ATTR_KEY_NAV_DOWN, ATTR_KEY_NAV_UP, Component, Dir, Flow, FlowableArgs,
    MeasurableComponent,
};
use crate::input::KeyMatcherQueryable;
use crate::{FrameContext, RenderArgs, UIResult};
use ratatui::layout::Size;
use ratatui::prelude::Style;
use ratatui::style::Color;
use ratatui::widgets::Block;
use std::ops::Neg;
use std::time::Instant;
use log::debug;

pub struct List<'a, ElementState, ElementOutput, Item, Element, CreateElement>
where
    ElementState: Default + 'static,
    ElementOutput: 'static,
    Element: MeasurableComponent<State = ElementState, Output = ElementOutput> + 'static,
    CreateElement: Fn(&Item, usize) -> Element,
{
    pub items: &'a Vec<Item>,
    pub create_element: CreateElement,
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
        }
    }
}

#[derive(Default)]
pub struct ListState {
    pub scroll_offset: i32,
    pub selection: usize,
}

impl<'a, ElementState, ElementOutput, Item, Element, CreateElement> Component
    for List<'a, ElementState, ElementOutput, Item, Element, CreateElement>
where
    ElementState: Default + 'static,
    ElementOutput: 'static,
    Element: MeasurableComponent<State = ElementState, Output = ElementOutput> + 'static,
    CreateElement: Fn(&Item, usize) -> Element,
{
    type State = ListState;
    type Output = ();

    fn render(&self, context: &FrameContext, state: &mut Self::State) -> UIResult<Self::Output> {
        struct ResolvedElement<Element> {
            key: String,
            element: Element,
            size: Size,
            index: usize,
        }

        let create_element = &self.create_element;
        let self_size = context.size();

        let mut resolved_elements: Vec<ResolvedElement<Element>> =
            Vec::with_capacity(self.items.len());
        let start = Instant::now();
        for (index, item) in self.items.iter().enumerate() {
            let element = create_element(item, index);
            let key = index.to_string();
            let size = context.measure_component(&key, &element)?;

            resolved_elements.push(ResolvedElement {
                key,
                element,
                size,
                index,
            });
        }
        debug!("resolve: {}", Instant::now().duration_since(start).as_secs_f64());

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
        state.selection = state.selection.max(0);
        debug!("input: {}", Instant::now().duration_since(start).as_secs_f64());

        // Check if we need to scroll to keep the selected item in view
        let selection_bottom_y: i32 = resolved_elements[0..state.selection]
            .iter()
            .map(|el| el.size.height as i32)
            .sum();

        // Move scroll offset if we're too far down
        state.scroll_offset += selection_bottom_y
            .saturating_add(1)
            .saturating_sub(state.scroll_offset)
            .saturating_sub(self_size.height as i32)
            .max(0);
        // Same but for up
        state.scroll_offset -= state.scroll_offset
            .saturating_sub(1)
            .saturating_sub(selection_bottom_y)
            .saturating_add(resolved_elements[state.selection].size.height as i32)
            .max(0);

        let mut current_y = -state.scroll_offset;

        for ResolvedElement {
            key,
            element,
            size,
            index,
        } in resolved_elements
        {
            // Render highlight for active selection
            if state.selection == index {
                context.render_widget(
                    Block::default().style(
                        Style::default()
                            .bg(context.req_attr::<Color>(ATTR_COLOR_HIGHLIGHT)?.clone()),
                    ),
                    (0, current_y).into(),
                    Size {
                        width: self_size.width,
                        height: size.height,
                    },
                );
            }

            context.render_component(
                RenderArgs::new(&element)
                    .key(&key)
                    .pos(0, current_y)
                    .size(self_size.width, size.height),
            )?;

            current_y += size.height as i32;
        }

        debug!("end: {}", Instant::now().duration_since(start).as_secs_f64());

        Ok(())
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
    fn measure(&self, context: &FrameContext, _: &Self::State) -> UIResult<Size> {
        let create_element = &self.create_element;
        let mut height = 0;
        let mut width = 0;

        for (index, item) in self.items.iter().enumerate() {
            let element = create_element(item, index);
            let size = context.measure_component(&index.to_string(), &element)?;
            height += size.height;
            width = size.width.max(width);
        }

        Ok(Size::new(width, height))
    }
}
