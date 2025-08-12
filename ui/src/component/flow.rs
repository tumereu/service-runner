use crate::component::{Component, MeasurableComponent};
use crate::space::Position;
use crate::{FrameContext, RenderArgs, SignalHandling, UIResult};
use ratatui::layout::Size;
use ratatui::style::{Color, Style};
use ratatui::widgets::Block;
use std::cmp::{max, min};

pub trait Flowable {
    fn measure(&self, ctx: &FrameContext, idx: usize) -> UIResult<Size>;
    // TODO output?
    fn render(&self, ctx: &mut FrameContext, idx: usize, pos: Position, size: Size) -> UIResult<()>;
}

impl<O, C: MeasurableComponent<Output = O>> Flowable for C
{
    fn measure(&self, ctx: &FrameContext, idx: usize) -> UIResult<Size> {
        ctx.measure_component(self)
    }

    fn render(&self, ctx: &mut FrameContext, idx: usize, pos: Position, size: Size) -> UIResult<()> {
        ctx.render_component(
            RenderArgs::new(self)
                .pos(pos.x, pos.y)
                .size(size.width, size.height)
                // TODO parameterize?
                .signals(SignalHandling::Forward)
        )?;

        Ok(())
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

    pub fn bg(mut self, bg: impl Into<Option<Color>>) -> Self {
        self.bg = bg.into();
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

    fn render(&self, context: &mut FrameContext) -> UIResult<Self::Output> {
        let self_size = context.size();
        if let Some(bg) = self.bg {
            context.render_widget(
                Block::default().style(Style::default().bg(bg)),
                (0, 0).into(),
                self_size,
            );
        }

        let mut free_space = if self.direction == Dir::UpDown {
            self_size.height
        } else {
            self_size.width
        };
        let mut num_fills = 0;

        for (idx, (cell, args)) in self.flowables.iter().enumerate() {
            free_space = free_space.saturating_sub(if args.fill {
                num_fills += 1;
                0
            } else {
                let measured_size = cell.measure(context, idx)?;

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

        for (idx, (flowable, args)) in self.flowables.iter().enumerate() {
            let measured_size = flowable.measure(context, idx)?;
            let size_in_layout: Size = (
                if self.direction == Dir::UpDown {
                    self_size.width
                } else if args.fill {
                    fill_size
                } else {
                    measured_size.width
                },
                if self.direction == Dir::LeftRight {
                    self_size.height
                } else if args.fill {
                    fill_size
                } else {
                    measured_size.height
                },
            )
                .into();

            // Clamp the size-in-layout to be a maximum of the remaining size
            let max_size: Size = match self.direction {
                Dir::UpDown => (self_size.width, self_size.height - current_pos).into(),
                Dir::LeftRight => (self_size.width - current_pos, self_size.height).into(),
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

            flowable.render(context, idx, (x, y).into(), size_in_layout)?;
        }

        Ok(())
    }
}
impl MeasurableComponent for Flow {
    fn measure(&self, context: &FrameContext) -> UIResult<Size> {
        let mut width: u16 = 0;
        let mut height: u16 = 0;

        for (idx, (flowable, _)) in self.flowables.iter().enumerate() {
            let child_size = flowable.measure(context, idx)?;

            if self.direction == Dir::UpDown {
                width = max(width, child_size.width);
                height += child_size.height;
            } else {
                width += child_size.width;
                height = max(height, child_size.height);
            }
        }

        Ok((width, height).into())
    }
}

#[derive(Eq, PartialEq, Default, Debug, Clone, Copy)]
pub enum Dir {
    #[default]
    LeftRight,
    UpDown,
}
