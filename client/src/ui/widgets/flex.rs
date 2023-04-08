use std::cmp::min;
use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;

use crate::ui::widgets::{Measured, Renderable, Size};
use crate::ui::widgets::FlexSize::Wrap;

pub struct Flex<B: Backend> {
    pub children: Vec<FlexElement<B>>,
    pub direction: FlexDir
}
impl<B: Backend> Renderable<B> for Flex<B> {
    fn render(self, rect: Rect, frame: &mut Frame<B>) {
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


            free_space = free_space.saturating_sub(
                match size {
                    FlexSize::Wrap => {
                        if self.direction == FlexDir::Column {
                            element.element.size.height
                        } else {
                            element.element.size.width
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
            let layout_width = match child.size_horiz {
                _ if self.direction == FlexDir::Column => rect.width,
                FlexSize::Wrap => child.element.size.width,
                FlexSize::Fixed(size) => size,
                FlexSize::Grow => grow_size,
            };
            let layout_height = match child.size_vert {
                _ if self.direction == FlexDir::Row => rect.height,
                FlexSize::Wrap => child.element.size.height,
                FlexSize::Fixed(size) => size,
                FlexSize::Grow => grow_size,
            };

            let (x, y) = if self.direction == FlexDir::Column {
                (
                    match child.align_horiz {
                        FlexAlign::Start => 0,
                        FlexAlign::End => layout_width - child.element.size.width,
                        FlexAlign::Center => (layout_width - child.element.size.width) / 2
                    },
                    current_pos
                )
            } else {
                (
                    current_pos,
                    match child.align_vert {
                        FlexAlign::Start => 0,
                        FlexAlign::End => layout_height - child.element.size.height,
                        FlexAlign::Center => (layout_height - child.element.size.height) / 2
                    },
                )
            };

            child.element.widget.render(
                Rect::new(
                    x,
                    y,
                    min(layout_width, child.element.size.width),
                    min(layout_height, child.element.size.height)
                ),
                frame
            );
        }
    }
}

pub struct FlexElement<B: Backend> {
    pub element: Measured<B>,
    pub size_horiz: FlexSize,
    pub size_vert: FlexSize,
    pub align_horiz: FlexAlign,
    pub align_vert: FlexAlign
}
impl<B: Backend> FlexElement<B> {
    fn from(element: Measured<B>) -> Self {
        FlexElement {
            element,
            size_horiz: Wrap,
            size_vert: Wrap,
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