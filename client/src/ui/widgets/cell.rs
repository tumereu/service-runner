use std::cell::RefCell;
use std::cmp::{max, min};
use std::fmt::format;
use std::rc::Rc;

use ratatui::backend::Backend;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders};
use ratatui::Frame;

use crate::ui::widgets::{Renderable, Size};

#[derive(Default, Debug)]
pub struct Cell {
    pub bg: Option<Color>,
    pub element: Option<Box<Renderable>>,
    pub border: Option<(Color, String)>,
    pub fill: bool,
    pub align_horiz: Align,
    pub align_vert: Align,
    pub padding_left: u16,
    pub padding_right: u16,
    pub padding_top: u16,
    pub padding_bottom: u16,
    pub min_width: u16,
    pub min_height: u16,
    pub store_bounds: Option<Rc<RefCell<Rect>>>,
}
impl Cell {
    pub fn render(self, rect: Rect, frame: &mut Frame)
    {
        if let Some(store_bounds) = self.store_bounds {
            store_bounds.replace(rect);
        }
        if self.border.is_some() || self.bg.is_some() {
            let mut block = Block::default();
            if let Some(bg) = self.bg {
                block = block.style(Style::default().bg(bg));
            } else {
                block = block.style(Style::default().bg(Color::Reset))
            }
            if let Some((color, title)) = &self.border {
                block = block
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(color.clone()))
                    .title(Span::from(title.to_string()));
            }
            frame.render_widget(block, rect);
        }

        let mut padding_left = self.padding_left;
        let mut padding_right = self.padding_right;
        let mut padding_top = self.padding_top;
        let mut padding_bottom = self.padding_bottom;

        if self.border.is_some() {
            padding_left += 1;
            padding_right += 1;
            padding_top += 1;
            padding_bottom += 1;
        }

        let el_size = self
            .element
            .as_ref()
            .map(|el| el.measure())
            .unwrap_or((0 as u16, 0 as u16).into());

        if let Some(element) = self.element {
            let max_width = rect.width.saturating_sub(padding_left + padding_right);
            let max_height = rect.height.saturating_sub(padding_top + padding_bottom);

            let width = if self.align_horiz == Align::Stretch {
                max_width
            } else {
                min(el_size.width, max_width)
            };
            let height = if self.align_vert == Align::Stretch {
                max_height
            } else {
                min(el_size.height, max_height)
            };

            let x = match self.align_horiz {
                Align::Start | Align::Stretch => rect.x + padding_left,
                Align::End => rect.x + rect.width - width - padding_right,
                Align::Center => rect.x + (rect.width - width) / 2,
            };
            let y = match self.align_vert {
                Align::Start | Align::Stretch => rect.y + padding_top,
                Align::End => rect.y + rect.height - height - padding_bottom,
                Align::Center => rect.y + (rect.height - height) / 2,
            };

            element.render(Rect::new(x, y, width, height), frame);
        }
    }

    pub fn measure(&self) -> Size {
        let el_size = self
            .element
            .as_ref()
            .map(|el| el.measure())
            .unwrap_or((0 as u16, 0 as u16).into());

        let border_pad = if self.border.is_some() {
            2
        } else {
            0
        };

        let mut width = el_size.width + self.padding_left + self.padding_right + border_pad;
        width = max(width, self.min_width);

        let mut height = el_size.height + self.padding_top + self.padding_bottom + border_pad;
        height = max(height, self.min_height);

        (width, height).into()
    }
}

#[derive(Eq, PartialEq, Default, Debug, Clone, Copy)]
pub enum Align {
    #[default]
    Start,
    End,
    Center,
    Stretch,
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

impl<R> IntoCell for R
where
    R: Into<Renderable>,
{
    fn into_el(self) -> Option<Box<Renderable>> {
        Some(Box::new(self.into()))
    }
}
