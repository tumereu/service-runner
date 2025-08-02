use std::cmp::max;

use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::Block;

use crate::ui::widgets::{Cell, Renderable, Size};

#[derive(Default, Debug)]
pub struct Flow {
    pub bg: Option<Color>,
    pub cells: Vec<Cell>,
    pub direction: Dir,
}
impl Flow {
    pub fn render<B>(self, rect: Rect, frame: &mut Frame<B>)
    where
        B: Backend,
    {
        if let Some(bg) = self.bg {
            frame.render_widget(Block::default().style(Style::default().bg(bg)), rect);
        }

        let mut free_space = if self.direction == Dir::UpDown {
            rect.height
        } else {
            rect.width
        };
        let mut num_fills = 0;

        for cell in &self.cells {
            free_space = free_space.saturating_sub(if cell.fill {
                num_fills += 1;
                0
            } else {
                let measured_size = cell.measure();

                if self.direction == Dir::UpDown {
                    measured_size.height
                } else {
                    measured_size.width
                }
            });
        }

        // TODO off-by-one errors? fix by moving inside loop and multiply by index or something?
        let fill_size = free_space / max(1, num_fills);
        let mut current_pos = 0;

        for cell in self.cells {
            let measured_size = cell.measure();
            let size_in_layout: Size = (
                if self.direction == Dir::UpDown {
                    rect.width
                } else if cell.fill {
                    fill_size
                } else {
                    measured_size.width
                },
                if self.direction == Dir::LeftRight {
                    rect.height
                } else if cell.fill {
                    fill_size
                } else {
                    measured_size.height
                },
            )
                .into();
            // Clamp the size-in-layout to be a maximum of the remaining size
            let size_in_layout = size_in_layout.intersect(match self.direction {
                Dir::UpDown => (rect.width, rect.height - current_pos).into(),
                Dir::LeftRight => (rect.width - current_pos, rect.height).into(),
            });

            let (x, y) = if self.direction == Dir::UpDown {
                (0, current_pos)
            } else {
                (current_pos, 0)
            };

            // Increase current position for subsequent elements
            current_pos += if self.direction == Dir::UpDown {
                size_in_layout.height
            } else {
                size_in_layout.width
            };

            cell.render(
                Rect::new(
                    rect.x + x,
                    rect.y + y,
                    size_in_layout.width,
                    size_in_layout.height,
                ),
                frame,
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
    UpDown,
}

impl From<Flow> for Renderable {
    fn from(value: Flow) -> Self {
        Renderable::Flow(value)
    }
}
