use std::cmp::{max, min};
use crate::component::{Align, Cell, Component, MeasurableComponent};
use crate::space::Position;
use crate::{FrameContext, RenderArgs, SignalHandling};
use ratatui::layout::{Rect, Size};
use ratatui::style::{Color, Style};
use ratatui::widgets::Block;

pub trait Flowable {
    fn measure(&self, ctx: &FrameContext, idx: usize) -> Size;
    // TODO output?
    fn render(&self, ctx: &FrameContext, idx: usize, pos: Position, size: Size);
}

impl<S: Default + 'static, O, C: MeasurableComponent<State = S, Output = O>> Flowable for C
{
    fn measure(&self, ctx: &FrameContext, idx: usize) -> Size {
        ctx.measure_component(&idx.to_string(), self)
    }

    fn render(&self, ctx: &FrameContext, idx: usize, pos: Position, size: Size) {
        ctx.render_component(
            RenderArgs::new(self)
                .pos(pos.x, pos.y)
                .size(size.width, size.height)
                // TODO parameterize?
                .signals(SignalHandling::Forward)
                .retain_unmounted_state(false)
                .key(&idx.to_string())
        );
    }
}

pub struct FlowableArgs {
    pub fill: bool,
}

#[derive(Default)]
pub struct Flow {
    pub bg: Option<Color>,
    pub flowables: Vec<(Box<dyn Flowable>, FlowableArgs)>,
    pub direction: Dir,
}
impl Flow {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bg(mut self, bg: Color) -> Self {
        self.bg = Some(bg);
        self
    }

    pub fn dir(mut self, direction: Dir) -> Self {
        self.direction = direction;
        self
    }

    pub fn element<F : Flowable + 'static>(mut self, flowable: F, args: FlowableArgs) -> Self {
        self.boxed_element(Box::new(flowable), args)
    }

    pub fn boxed_element(mut self, flowable: Box<dyn Flowable>, args: FlowableArgs) -> Self {
        self.flowables.push((flowable, args));
        self
    }
}

impl Component for Flow {
    type Output = ();
    type State = ();

    fn render(&self, context: &FrameContext, _: &mut Self::State) -> Self::Output {
        let area = context.area();
        if let Some(bg) = self.bg {
            context.render_widget(Block::default().style(Style::default().bg(bg)), area);
        }

        let mut free_space = if self.direction == Dir::UpDown {
            area.height
        } else {
            area.width
        };
        let mut num_fills = 0;

        for (idx, (cell, args)) in self.flowables.iter().enumerate() {
            free_space = free_space.saturating_sub(if args.fill {
                num_fills += 1;
                0
            } else {
                let measured_size = cell.measure(context, idx);

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

        for (idx, (cell, args)) in self.flowables.iter().enumerate() {
            let measured_size = cell.measure(context, idx);
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
            let max_size: Size = match self.direction {
                Dir::UpDown => (area.width, area.height - current_pos).into(),
                Dir::LeftRight => (area.width - current_pos, area.height).into(),
            };
            let size_in_layout: Size = (
                min(size_in_layout.width, max_size.width),
                min(size_in_layout.height, max_size.height),
            )
                .into();

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

            cell.render(context, idx, (x, y).into(), size_in_layout);
        }
    }
}
impl MeasurableComponent for Flow {
    fn measure(&self, context: &FrameContext, _: &Self::State) -> Size {
        let mut width: u16 = 0;
        let mut height: u16 = 0;

        for (idx, (cell, _)) in self.flowables.iter().enumerate() {
            let child_size = cell.measure(context, idx);

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
