use std::cmp::max;

use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::{List as TuiList, ListItem as TuiListItem};

use crate::ui::widgets::{Flex, FlexDir, FlexElement, FlexSize, IntoFlexElement, Renderable, Size, Styleable, Text};

pub struct List {
    items: Vec<Renderable>,
    selection: usize,
}
impl List {
    pub fn new() -> List {
        List {
            items: Vec::new(),
            selection: 0,
        }
    }

    pub fn simple_items(self, items: Vec<String>) -> Self {
        List {
            items: items.into_iter()
                .map(|item| Text::new(item).into())
                .collect(),
            ..self
        }
    }

    pub fn items(self, items: Vec<Renderable>) -> Self {
        List {
            items,
            ..self
        }
    }

    pub fn selection(self, selection: usize) -> Self {
        List {
            selection,
            ..self
        }
    }

    pub fn render<B>(self, rect: Rect, frame: &mut Frame<B>) where B: Backend {
        let mut items: Vec<FlexElement> = self.items.into_iter()
            .enumerate()
            .map(|(index, item)| {
                if self.selection == index {
                    item.styling().bg(Color::Blue).into()
                } else {
                    item
                }.into_flex().grow_horiz()
            }).collect();

        Flex::new(items)
            .direction(FlexDir::UpDown)
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
