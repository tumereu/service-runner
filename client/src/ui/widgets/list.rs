use std::cmp::max;

use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::{List as TuiList, ListItem as TuiListItem};

use crate::ui::widgets::{CellLayout, Dir, Cell, Align, Renderable, Size, Text, IntoCell};

#[derive(Debug, Default)]
pub struct List {
    pub items: Vec<Cell>,
    pub selection: usize,
}
impl List {
    pub fn simple_items(items: Vec<String>) -> Vec<Cell> {
        items.into_iter()
            .map(|item| {
                Cell {
                    element: Text {
                        text: item,
                        ..Default::default()
                    }.into_el(),
                    ..Default::default()
                }
            })
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
                }
            }).collect();

        CellLayout {
            cells: items,
            direction: Dir::UpDown,
            ..Default::default()
        }.render(rect, frame);
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
