use crate::component::{Component, MeasurableComponent, StatefulComponent, ATTR_COLOR_HIGHLIGHT};
use crate::{FrameContext, RenderArgs, UIError, UIResult};
use ratatui::layout::Size;
use ratatui::prelude::Style;
use ratatui::style::Color;
use ratatui::widgets::Block;

pub struct List<'a, ElementOutput, Item, Element, CreateElement>
where
    ElementOutput: 'static,
    Element: MeasurableComponent<Output = ElementOutput> + 'static,
    CreateElement: Fn(&Item, usize) -> UIResult<Element>,
{
    pub items: &'a Vec<Item>,
    pub create_element: CreateElement,
    pub id: String,
    pub selection: Option<usize>,
}
impl<'a, ElementOutput, Item, Element, CreateElement>
    List<'a, ElementOutput, Item, Element, CreateElement>
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
            selection: None,
        }
    }
    
    pub fn selection(mut self, selection: Option<usize>) -> Self {
        self.selection = selection;
        self
    }
}

#[derive(Default)]
pub struct ListState {
    pub scroll_offset: i32,
}

impl<'a, ElementOutput, Item, Element, CreateElement> StatefulComponent
    for List<'a, ElementOutput, Item, Element, CreateElement>
where
    ElementOutput: 'static,
    Element: MeasurableComponent<Output = ElementOutput> + 'static,
    CreateElement: Fn(&Item, usize) -> UIResult<Element>,
{
    type State = ListState;
    type Output = ();

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
        let selection = self.selection.unwrap_or_default().max(0).min(resolved_elements.len() - 1);

        // Check if we need to scroll to keep the selected item in view
        let selection_bottom_y: i32 = resolved_elements[0..selection]
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
            .saturating_add(resolved_elements[selection].size.height as i32)
            .max(0);

        let mut current_y = -state.scroll_offset;

        for ResolvedElement {
            element,
            size,
            index,
        } in resolved_elements
        {
            // Render highlight for active selection
            if self.selection.is_some() && selection == index {
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
                RenderArgs::new(element)
                    .pos(0, current_y)
                    .size(self_size.width, size.height),
            )?;

            current_y += size.height as i32;
        }
        
        Ok(())
    }
}

impl<'a, ElementOutput, Item, Element, CreateElement> MeasurableComponent
    for List<'a, ElementOutput, Item, Element, CreateElement>
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
