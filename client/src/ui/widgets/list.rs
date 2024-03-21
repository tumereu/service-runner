use std::cmp::max;

use ratatui::backend::Backend;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::Frame;

use crate::ui::widgets::{Align, Cell, Dir, Flow, IntoCell, Renderable, Size, Text};

#[derive(Debug, Default)]
pub struct List {
    pub items: Vec<Cell>,
    pub selection: usize,
}
impl List {
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
            .into()
    }
}

impl From<List> for Renderable {
    fn from(value: List) -> Self {
        Renderable::List(value)
    }
}
