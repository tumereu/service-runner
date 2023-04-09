use std::cmp::max;

use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::text::Text;
use tui::widgets::{List as TuiList, ListItem as TuiListItem};

use crate::ui::widgets::{Renderable, Size};

pub struct List {
    items: Vec<String>,
    selection: usize,
}
impl List {
    pub fn new() -> List {
        List {
            items: Vec::new(),
            selection: 0,
        }
    }

    pub fn items(self, items: Vec<String>) -> Self {
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

    pub fn render<B>(&self, rect: Rect, frame: &mut Frame<B>) where B: Backend {
        let list = TuiList::new(
            self.items.iter()
                .enumerate()
                .map(|(index, item)| {
                    TuiListItem::new(Text::from(item.clone()))
                        .style(
                            if self.selection == index {
                                Style::default()
                                    .bg(Color::Rgb(204, 153, 0))
                            } else {
                                Style::default()
                            }
                        )
                }).collect::<Vec<TuiListItem>>()
        );

        frame.render_widget(list, rect);
    }

    pub fn measure(&self) -> Size {
        (
            self.items.iter()
                .map(|item| item.len() as u16)
                .reduce(|a, b| max(a, b))
                .unwrap_or(0),
            self.items.len() as u16
        ).into()
    }
}

impl From<List> for Renderable {
    fn from(value: List) -> Self {
        Renderable::List(value)
    }
}
