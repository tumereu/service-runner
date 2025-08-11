use std::cmp::max;

use ratatui::layout::{Rect, Size};
use ratatui::style::{Color, Style};
use ratatui::widgets::Block;
use crate::component::{Align, Cell, Component, MeasurableComponent};
use crate::FrameContext;

pub trait FlowCell {
    fn get_align_vert(&self) -> Align;
    fn get_align_horiz(&self) -> Align;
    fn measure(&self, idx: usize, ctx: &FrameContext) -> Size;
}

impl<S : Default + 'static, O, C : MeasurableComponent<State = S, Output = O>> FlowCell for Cell<S, O, C> {
    fn get_align_vert(&self) -> Align {
        self.align_vert
    }

    fn get_align_horiz(&self) -> Align {
        self.align_horiz
    }
    
    fn measure(&self, idx: usize, ctx: &FrameContext) -> Size {
        ctx.measure_component(&idx.to_string(), self)
    }
}

pub struct CellArgs {
    pub fill: bool,
}

#[derive(Default)]
pub struct Flow {
    pub bg: Option<Color>,
    pub cells: Vec<(Box<dyn FlowCell>, CellArgs)>,
    pub direction: Dir,
}
impl Component for Flow {
    type Output = ();
    type State = ();

    fn render(&self, context: &FrameContext, _: &mut Self::State) -> Self::Output
    {
        let area = context.area();
        if let Some(bg) = self.bg {
            context.render_widget(
                Block::default().style(Style::default().bg(bg)), 
                area,
            );
        }

        let mut free_space = if self.direction == Dir::UpDown {
            area.height
        } else {
            area.width
        };
        let mut num_fills = 0;

        for (idx, (cell, args)) in self.cells.iter().enumerate() {
            free_space = free_space.saturating_sub(if args.fill {
                num_fills += 1;
                0
            } else {
                let measured_size = cell.measure(idx, context);

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

        for (idx, (cell, args)) in self.cells.iter().enumerate() {
            let measured_size = cell.measure(idx, context);
            let size_in_layout: Size = (
                if self.direction == Dir::UpDown {
                    area.width
                } else if args.fill {
                    fill_size
                } else {
                    measured_size.width
                },
                if self.direction == Dir::LeftRight {
                    area.height
                } else if args.fill {
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
