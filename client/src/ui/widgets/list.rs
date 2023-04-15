use std::cmp::max;

use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::{List as TuiList, ListItem as TuiListItem};

use crate::ui::widgets::{Container, CellLayout, Dir, Cell, Align, IntoFlexElement, Renderable, Size, Text};

#[derive(Debug, Default)]
pub struct List {
    pub items: Vec<Cell>,
    pub selection: usize,
}
impl List {
    pub fn simple_items(items: Vec<String>) -> Vec<Renderable> {
        items.into_iter()
            .map(|item| Text::new(item).into())
            .collect()
    }

    pub fn render<B>(self, rect: Rect, frame: &mut Frame<B>) where B: Backend {
        let mut items: Vec<Cell> = self.items.into_iter()
            .enumerate()
            .map(|(index, item)| {
                if self.selection == index {
                    Cell {
                        bg: Some(Color::Blue),
                        ..item
                    }
                } else {
                    item
                }.into_flex().grow_horiz()
            }).collect();

        CellLayout::new(items)
            .direction(Dir::UpDown)
            .render(rect, frame);
    }

    pub fn measure(&self) -> Size {
        self.items.iter()
            .map(|item| item.measure())
            .reduce(|a, b| {
                (
                    max(a.width, b.width),
                    a.height + b.height
                ).into()
            }).unwrap_or(Size::empty()).into()
    }
}

impl From<List> for Renderable {
    fn from(value: List) -> Self {
        Renderable::List(value)
    }
}
