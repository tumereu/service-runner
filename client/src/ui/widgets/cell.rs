use std::cmp::max;
use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::Block;
use crate::ui::widgets::{Renderable, Size};

#[derive(Default, Debug)]
pub struct Cell {
    pub bg: Option<Color>,
    pub element: Option<Box<Renderable>>,
    pub fill: bool,
    pub align_horiz: Align,
    pub align_vert: Align,
    pub padding_left: u16,
    pub padding_right: u16,
    pub padding_top: u16,
    pub padding_bottom: u16,
    pub min_width: u16,
    pub min_height: u16,
}
impl Cell {
    pub fn render<B>(self, rect: Rect, frame: &mut Frame<B>) where B: Backend {
        if let Some(bg) = self.bg {
            frame.render_widget(
                Block::default().style(Style::default().bg(bg)),
                rect
            );
        }

        let el_size = self.measure();
        if let Some(element) = self.element {
            let width = if self.align_horiz == Align::Stretch {
                rect.width
            } else {
                el_size.width - self.padding_left - self.padding_right
            };
            let height = if self.align_vert == Align::Stretch {
                rect.height
            } else {
                el_size.height - self.padding_top - self.padding_bottom
            };

            let x = match self.align_horiz {
                Align::Start | Align::Stretch => rect.x + self.padding_left,
                Align::End => rect.x + rect.width - el_size.width - self.padding_right,
                Align::Center => rect.x + (rect.width - width) / 2 + self.padding_left,
            };
            let y = match self.align_vert {
                Align::Start | Align::Stretch => rect.y + self.padding_top,
                Align::End => rect.y + rect.height - el_size.height - self.padding_bottom,
                Align::Center => rect.y + (rect.height - height) / 2 + self.padding_top,
            };

            element.render(Rect::new(x, y, width, height), frame);
        }
    }

    pub fn measure(&self) -> Size {
        let el_size = self.element.as_ref().map(|el| el.measure()).unwrap_or((0 as u16, 0 as u16).into());

        let mut width = el_size.width + self.padding_left + self.padding_right;
        width = max(width, self.min_width);

        let mut height = el_size.height + self.padding_top + self.padding_bottom;
        height = max(height, self.min_height);

        (width, height).into()
    }
}

#[derive(Eq, PartialEq, Default, Debug)]
pub enum Align {
    #[default]
    Start,
    End,
    Center,
    Stretch
}

impl From<Cell> for Renderable {
    fn from(value: Cell) -> Self {
        Renderable::Cell(value)
    }
}

/// A helper trait that can be used to convert elements into the correct type required by [Cell] when passing them as
/// a value for the element-field.
pub trait IntoCell {
    fn into_el(self) -> Option<Box<Renderable>>;
}

impl<R> IntoCell for R where R : Into<Renderable> {
    fn into_el(self) -> Option<Box<Renderable>> {
        Some(Box::new(self.into()))
    }
}