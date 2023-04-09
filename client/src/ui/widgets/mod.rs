use std::cmp::max;
use tui::backend::Backend;
use tui::Frame;
use tui::widgets::{List as TuiList, ListItem as TuiListItem};
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::text::Text;
pub use flex::*;
pub use size::*;

mod size;
mod flex;

pub enum Renderable {
    Flex(Flex),
    List {
        items: Vec<String>,
        selection: usize,
        size: Size
    }
}
impl Renderable {
    fn render<B>(self, rect: Rect, frame: &mut Frame<B>) where B: Backend {
        match self {
            Renderable::Flex(flex) => flex.render(rect, frame),
            Renderable::List { items, selection, .. } => {
                render_list(items, selection, rect, frame);
            }
        }
    }

    fn measure(&self) -> Size {
        match self {
            Renderable::Flex(flex) => flex.measure(),
            Renderable::List { items, .. } => {
                (
                    items.iter()
                        .map(|item| item.len() as u16)
                        .reduce(|a, b| max(a, b))
                        .unwrap_or(0),
                    items.len() as u16
                ).into()
            }
        }
    }
}

fn render_list<B>(items: Vec<String>, selection: usize, rect: Rect, frame: &mut Frame<B>) where B: Backend {
    let list = TuiList::new(
        items.into_iter()
            .enumerate()
            .map(|(index, item)| {
                TuiListItem::new(Text::from(item))
                    .style(
                        if selection == index {
                            Style::default()
                                .bg(Color::White)
                                .fg(Color::Black)
                        } else {
                            Style::default()
                                .bg(Color::Black)
                                .fg(Color::White)
                        }
                    )
            }).collect::<Vec<TuiListItem>>()
    );

    frame.render_widget(list, rect);
}