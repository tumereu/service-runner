use std::cmp::{max, min};
use std::ops::Deref;

use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;

use crate::ui::widgets::{Renderable, Size};

pub struct Flex {
    pub children: Vec<FlexElement>,
    pub direction: FlexDir
}
impl Flex {
    pub fn new() -> Flex {
        Flex {
            children: Vec::new(),
            direction: FlexDir::Column
        }
    }

    pub fn children(self, children: Vec<FlexElement>) -> Self {
        Flex {
            children,
            ..self
        }
    }

    pub fn direction(self, direction: FlexDir) -> Self {
        Flex {
            direction,
            ..self
        }
    }

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
        let grow_size = free_space / max(1, num_grows);
        let padding = if num_grows == 0 {
            free_space / min(1, self.children.len() as u16)
        } else {
            0
        };
        let mut current_pos = 0;

        for child in self.children {
            let measured_size = child.renderable.measure();
            let size_in_layout: Size = (
                match child.size_horiz {
                    _ if self.direction == FlexDir::Column => rect.width,
                    FlexSize::Wrap => measured_size.width + padding,
                    FlexSize::Fixed(size) => size + padding,
                    FlexSize::Grow => grow_size,
                },
                match child.size_vert {
                    _ if self.direction == FlexDir::Row => rect.height,
                    FlexSize::Wrap => measured_size.height + padding,
                    FlexSize::Fixed(size) => size + padding,
                    FlexSize::Grow => grow_size,
                }
            ).into();
            // Clamp the size-in-layout to be a maximum of the remaining size
            let size_in_layout = size_in_layout.intersect(
                match self.direction {
                    FlexDir::Column => (rect.width, rect.height - current_pos).into(),
                    FlexDir::Row => (rect.width - current_pos, rect.height).into()
                }
            );

            let actual_size = measured_size.intersect(size_in_layout);

            let (x, y) = if self.direction == FlexDir::Column {
                (
                    match child.align_horiz {
                        FlexAlign::Start => 0,
                        FlexAlign::End => rect.width - actual_size.width,
                        FlexAlign::Center => (rect.width - actual_size.width) / 2
                    },
                    current_pos + match child.align_vert {
                        FlexAlign::Start => 0,
                        FlexAlign::End => size_in_layout.height - actual_size.height,
                        FlexAlign::Center => (size_in_layout.height - actual_size.height) / 2
                    }
                )
            } else {
                (
                    current_pos + match child.align_horiz {
                        FlexAlign::Start => 0,
                        FlexAlign::End => size_in_layout.width - actual_size.width,
                        FlexAlign::Center => (size_in_layout.width - actual_size.width) / 2
                    },
                    match child.align_vert {
                        FlexAlign::Start => 0,
                        FlexAlign::End => rect.height - actual_size.height,
                        FlexAlign::Center => (rect.height - actual_size.height) / 2
                    },
                )
            };

            // Increase current position for subseqent elements
            current_pos += if self.direction == FlexDir::Column {
                size_in_layout.height
            } else {
                size_in_layout.width
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
    pub fn from<R>(element: R) -> Self where R: Into<Renderable> {
        FlexElement {
            renderable: element.into(),
            size_horiz: FlexSize::Wrap,
            size_vert: FlexSize::Wrap,
            align_horiz: FlexAlign::Center,
            align_vert: FlexAlign::Center
        }
    }

    pub fn size_horiz(self, size_horiz: FlexSize) -> Self {
        FlexElement {
            size_horiz,
            ..self
        }
    }

    pub fn size_vert(self, size_vert: FlexSize) -> Self {
        FlexElement {
            size_vert,
            ..self
        }
    }

    pub fn align_horiz(self, align_horiz: FlexAlign) -> Self {
        FlexElement {
            align_horiz,
            ..self
        }
    }

    pub fn align_vert(self, align_vert: FlexAlign) -> Self {
        FlexElement {
            align_vert,
            ..self
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

impl From<Flex> for Renderable {
    fn from(value: Flex) -> Self {
        Renderable::Flex(value)
    }
}