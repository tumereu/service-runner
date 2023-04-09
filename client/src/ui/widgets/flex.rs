use std::cmp::{max, min};
use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;

use crate::ui::widgets::{Renderable, Size};

pub struct Flex {
    pub children: Vec<FlexElement>,
    pub direction: FlexDir
}
impl Flex {
    pub fn render<B>(self, rect: Rect, frame: &mut Frame<B>) where B: Backend {
        let mut free_space = if self.direction == FlexDir::Column {
            rect.height
        } else {
            rect.height
        };
        let mut num_grows = 0;

        for element in &self.children {
            let size = if self.direction == FlexDir::Column {
                &element.size_vert
            } else {
                &element.size_horiz
            };
            let measured_size = element.renderable.measure();

            free_space = free_space.saturating_sub(
                match size {
                    FlexSize::Wrap => {
                        if self.direction == FlexDir::Column {
                            measured_size.height
                        } else {
                            measured_size.width
                        }
                    },
                    FlexSize::Fixed(size) => *size,
                    FlexSize::Grow => {
                        num_grows += 1;
                        0
                    }
                }
            );
        }

        // TODO off-by-one errors?
        let grow_size = free_space / num_grows;
        let mut current_pos = 0;

        for child in self.children {
            let measured_size = child.renderable.measure();
            let layout_size: Size = (
                match child.size_horiz {
                    _ if self.direction == FlexDir::Column => rect.width,
                    FlexSize::Wrap => measured_size.width,
                    FlexSize::Fixed(size) => size,
                    FlexSize::Grow => grow_size,
                },
                match child.size_vert {
                    _ if self.direction == FlexDir::Row => rect.height,
                    FlexSize::Wrap => measured_size.height,
                    FlexSize::Fixed(size) => size,
                    FlexSize::Grow => grow_size,
                }
            ).into();
            let actual_size = measured_size.intersect(&layout_size);

            let (x, y) = if self.direction == FlexDir::Column {
                (
                    match child.align_horiz {
                        FlexAlign::Start => 0,
                        FlexAlign::End => layout_size.width - actual_size.width,
                        FlexAlign::Center => (layout_size.width - actual_size.width) / 2
                    },
                    current_pos
                )
            } else {
                (
                    current_pos,
                    match child.align_vert {
                        FlexAlign::Start => 0,
                        FlexAlign::End => layout_size.height - actual_size.height,
                        FlexAlign::Center => (layout_size.height - actual_size.height) / 2
                    },
                )
            };

            child.renderable.render(
                Rect::new(
                    x,
                    y,
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

        for child in &self.children {
            let child_size = child.renderable.measure();

            // TODO what about grow-elements?
            if self.direction == FlexDir::Column {
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

pub struct FlexElement {
    pub renderable: Renderable,
    pub size_horiz: FlexSize,
    pub size_vert: FlexSize,
    pub align_horiz: FlexAlign,
    pub align_vert: FlexAlign
}
impl FlexElement {
    fn from(element: Renderable) -> Self {
        FlexElement {
            renderable: element,
            size_horiz: FlexSize::Wrap,
            size_vert: FlexSize::Wrap,
            align_horiz: FlexAlign::Center,
            align_vert: FlexAlign::Center
        }
    }
}

#[derive(Eq, PartialEq)]
pub enum FlexSize {
    Fixed(u16),
    Wrap,
    Grow
}

#[derive(Eq, PartialEq)]
pub enum FlexAlign {
    Start,
    End,
    Center
}

#[derive(Eq, PartialEq)]
pub enum FlexDir {
    Row,
    Column
}