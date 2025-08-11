use std::cmp::max;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::Frame;
use crate::component::{Cell, Component, Flow, MeasurableComponent};
use crate::{FrameContext, RenderArgs};

pub struct VList<ElementState, ElementOutput, Item, Element, CreateElement>
where
    Element: MeasurableComponent<State=ElementState, Output=ElementOutput>,
    CreateElement: Fn(&Item, usize) -> Element,
{
    pub items: Vec<Item>,
    pub selection: usize,
    pub create_element: CreateElement,
}
impl<ElementState, ElementOutput, Item, Element, CreateElement> VList<ElementState, ElementOutput, Item, Element, CreateElement> {
    fn create_flow(&self) -> Flow {
        let mut flow = Flow::new();

        for (index, item) in self.items.iter().enumerate() {
            let cell = Cell::containing(self.render_item(item, index));
            flow = flow.element(

            )
        }

        flow
    }
}

impl<ElementState, ElementOutput, Item, Element, CreateElement> Component for VList<ElementState, ElementOutput, Item, Element, CreateElement>
where
    Element: MeasurableComponent<State=ElementState, Output=ElementOutput>,
    CreateElement: Fn(&Item, usize) -> Element
{
    type State = ();
    type Output = ();

    fn render(&self, context: &FrameContext, _: &mut Self::State) -> Self::Output {
        let mut flow = Flow::new();

        context.render_component(
            RenderArgs::new(
                Flow::new()

            )
        )
    }
}


pub fn simple_items(items: Vec<String>, align_horiz: Align) -> Vec<Cell> {
    items
        .into_iter()
        .map(|item| Cell {
            align_horiz,
            element: Text {
                text: item,
                ..Default::default()
            }
                .into_el(),
            ..Default::default()
        })
        .collect()
}

pub fn render(self, rect: Rect, frame: &mut Frame)
{
    let items: Vec<Cell> = self
        .items
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            if self.selection == index {
                Cell {
                    bg: Some(Color::Blue),
                    ..item
                }
            } else {
                item
            }
        })
        .collect();

    Flow {
        cells: items,
        direction: Dir::UpDown,
        ..Default::default()
    }
        .render(rect, frame);
}

pub fn measure(&self) -> Size {
    self.items
        .iter()
        .map(|item| item.measure())
        .reduce(|a, b| (max(a.width, b.width), a.height + b.height).into())
        .unwrap_or(Size::empty())
}
}

impl From<List> for Renderable {
    fn from(value: List) -> Self {
        Renderable::List(value)
    }
}
