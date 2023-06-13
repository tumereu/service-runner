use std::cmp::max;
use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;
use tui::style::Color;
use crate::ui::widgets::{Cell, Dir, Flow, IntoCell, Renderable, Size, Text};

#[derive(Debug, Default)]
pub struct Toggle {
    pub options: Vec<String>,
    pub selection: usize,
}
impl Toggle {
    pub fn render<B>(self, rect: Rect, frame: &mut Frame<B>)
        where
            B: Backend,
    {
        let items: Vec<Cell> = self
            .options
            .into_iter()
            .enumerate()
            .map(|(index, item)| {
                Cell {
                    bg: if self.selection == index {
                        Some(Color::Blue)
                    } else {
                        None
                    },
                    fill: true,
                    element: Text {
                        text: format!(" {item} "),
                        ..Default::default()
                    }.into_el(),
                    ..Default::default()
                }

            })
            .collect();

        Flow {
            cells: items,
            direction: Dir::LeftRight,
            ..Default::default()
        }
            .render(rect, frame);
    }

    pub fn measure(&self) -> Size {
        let max_width: u16 = self.options.iter().map(|item| item.len() as u16 + 2).max().unwrap_or(0);
        // Reserve even horizontal space for all texts
        Size {
            width: max_width.saturating_mul(self.options.len() as u16),
            height: 1
        }
    }
}

impl From<Toggle> for Renderable {
    fn from(value: Toggle) -> Self {
        Renderable::Toggle(value)
    }
}
