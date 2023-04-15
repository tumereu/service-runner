use std::cmp::{max, min};

use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::Block;

use crate::ui::widgets::{Align, Cell, Renderable, Size};

#[derive(Default, Debug)]
pub struct CellLayout {
    pub bg: Option<Color>,
    pub cells: Vec<Cell>,
    pub direction: Dir,
}
impl CellLayout {
    pub fn render<B>(self, rect: Rect, frame: &mut Frame<B>) where B: Backend {
        if let Some(bg) = self.bg {
            frame.render_widget(
                Block::default().style(Style::default().bg(bg)),
                rect
            );
        }

        let mut free_space = if self.direction == Dir::UpDown {
            rect.height
        } else {
            rect.width
        };
        let mut num_fills = 0;

        for cell in &self.cells {
            let align = if self.direction == Dir::UpDown {
                &cell.align_vert
            } else {
                &cell.align_horiz
            };
            let measured_size = cell.element.measure();

            free_space = free_space.saturating_sub(
                if cell.fill {
                    num_fills += 1;
                    0
                } else {
                    if self.direction == Dir::UpDown {
                        measured_size.height
                    } else {
                        measured_size.width
                    }
                }
            );
        }

        // TODO off-by-one errors? fix by moving inside loop and multiply by index or something?
        let fill_size = free_space / max(1, num_fills);
        let mut current_pos = 0;

        for cell in self.cells {
            let measured_size = cell.element.measure();
            let size_in_layout: Size = (
                match cell.aloi {
                    _ if self.direction == Dir::UpDown => rect.width,
                    Align::Wrap => measured_size.width,
                    Align::Grow => fill_size,
                },
                match cell.size_vert {
                    _ if self.direction == Dir::LeftRight => rect.height,
                    Align::Wrap => measured_size.height,
                    Align::Grow => fill_size,
                }
            ).into();
            // Clamp the size-in-layout to be a maximum of the remaining size
            let size_in_layout = size_in_layout.intersect(
                match self.direction {
                    Dir::UpDown => (rect.width, rect.height - current_pos).into(),
                    Dir::LeftRight => (rect.width - current_pos, rect.height).into()
                }
            );

            let actual_size: Size = (
                match cell.size_horiz {
                    Align::Wrap => measured_size.width,
                    Align::Grow => match self.direction {
                        Dir::UpDown => rect.width,
                        Dir::LeftRight => fill_size
                    },
                },
                match cell.size_vert {
                    Align::Wrap => measured_size.height,
                    Align::Grow => match self.direction {
                        Dir::UpDown => fill_size,
                        Dir::LeftRight => rect.height
                    },
                },
            ).into();
            // Clamp the actual size to a maximum of the size in layout
            let actual_size = actual_size.intersect(size_in_layout);

            let (x, y) = if self.direction == Dir::UpDown {
                (0, current_pos)
            } else {
                (current_pos, 0)
            };

            // Increase current position for subseqent elements
            current_pos += if self.direction == Dir::UpDown {
                size_in_layout.height
            } else {
                size_in_layout.width
            };

            cell.element.render(
                Rect::new(
                    rect.x + x,
                    rect.y + y,
                    actual_size.width,
                    actual_size.height
                ),
                frame
            );
        }
    }

    pub fn measure(&self) -> Size {
        let mut width: u16 = 0;
        let mut height: u16 = 0;

        for cell in &self.cells {
            let child_size = cell.measure();

            if self.direction == Dir::UpDown {
                width = max(width, child_size.width);
                height += child_size.height;
            } else {
                width += child_size.width;
                height = max(height, child_size.height);
            }
        }

        (width, height).into()
    }
}

#[derive(Eq, PartialEq, Default, Debug)]
pub enum Dir {
    #[default]
    LeftRight,
    UpDown
}

impl From<CellLayout> for Renderable {
    fn from(value: CellLayout) -> Self {
        Renderable::CellLayout(value)
    }
}