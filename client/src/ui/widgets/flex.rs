use std::cmp::{max, min};

use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::Block;

use crate::ui::widgets::{Renderable, Size};

pub struct Flex {
    bg: Option<Color>,
    children: Vec<FlexElement>,
    direction: FlexDir,
}
impl Flex {
    pub fn new(children: Vec<FlexElement>) -> Flex {
        Flex {
            children,
            bg: None,
            direction: FlexDir::UpDown,
        }
    }

    pub fn direction(self, direction: FlexDir) -> Flex {
        Flex {
            direction,
            ..self
        }
    }

    pub fn bg(self, bg: Option<Color>) -> Flex {
        Flex {
            bg,
            ..self
        }
    }

    pub fn render<B>(self, rect: Rect, frame: &mut Frame<B>) where B: Backend {
        if let Some(bg) = self.bg {
            frame.render_widget(
                Block::default().style(Style::default().bg(bg)),
                rect
            );
        }

        let mut free_space = if self.direction == FlexDir::UpDown {
            rect.height
        } else {
            rect.width
        };
        let mut num_grows = 0;

        for element in &self.children {
            let size = if self.direction == FlexDir::UpDown {
                &element.size_vert
            } else {
                &element.size_horiz
            };
            let measured_size = element.renderable.measure();

            free_space = free_space.saturating_sub(
                match size {
                    FlexSize::Wrap => {
                        if self.direction == FlexDir::UpDown {
                            measured_size.height
                        } else {
                            measured_size.width
                        }
                    },
                    FlexSize::Grow => {
                        num_grows += 1;
                        0
                    }
                }
            );
        }

        // TODO off-by-one errors? fix by moving inside loop and multiply by index or something?
        let grow_size = free_space / max(1, num_grows);
        let mut current_pos = 0;

        for child in self.children {
            let measured_size = child.renderable.measure();
            let size_in_layout: Size = (
                match child.size_horiz {
                    _ if self.direction == FlexDir::UpDown => rect.width,
                    FlexSize::Wrap => measured_size.width,
                    FlexSize::Grow => grow_size,
                },
                match child.size_vert {
                    _ if self.direction == FlexDir::LeftRight => rect.height,
                    FlexSize::Wrap => measured_size.height,
                    FlexSize::Grow => grow_size,
                }
            ).into();
            // Clamp the size-in-layout to be a maximum of the remaining size
            let size_in_layout = size_in_layout.intersect(
                match self.direction {
                    FlexDir::UpDown => (rect.width, rect.height - current_pos).into(),
                    FlexDir::LeftRight => (rect.width - current_pos, rect.height).into()
                }
            );

            let actual_size: Size = (
                match child.size_horiz {
                    FlexSize::Wrap => measured_size.width,
                    FlexSize::Grow => match self.direction {
                        FlexDir::UpDown => rect.width,
                        FlexDir::LeftRight => grow_size
                    },
                },
                match child.size_vert {
                    FlexSize::Wrap => measured_size.height,
                    FlexSize::Grow => match self.direction {
                        FlexDir::UpDown => grow_size,
                        FlexDir::LeftRight => rect.height
                    },
                },
            ).into();
            // Clamp the actual size to a maximum of the size in layout
            let actual_size = actual_size.intersect(size_in_layout);

            let (x, y) = if self.direction == FlexDir::UpDown {
                (0, current_pos)
            } else {
                (current_pos, 0)
            };

            // Increase current position for subseqent elements
            current_pos += if self.direction == FlexDir::UpDown {
                size_in_layout.height
            } else {
                size_in_layout.width
            };

            child.renderable.render(
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

        for child in &self.children {
            let child_size = child.renderable.measure();

            // TODO what about grow-elements?
            if self.direction == FlexDir::UpDown {
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
    renderable: Renderable,
    size_horiz: FlexSize,
    size_vert: FlexSize,
}
impl FlexElement {
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

    pub fn grow_vert(self) -> Self {
        self.size_vert(FlexSize::Grow)
    }

    pub fn grow_horiz(self) -> Self {
        self.size_horiz(FlexSize::Grow)
    }

    pub fn grow_both(self) -> Self {
        self.grow_vert().grow_horiz()
    }
}

pub trait IntoFlexElement {
    fn into_flex(self) -> FlexElement;
}

impl<R : Into<Renderable>> IntoFlexElement for R {
    fn into_flex(self) -> FlexElement {
        FlexElement {
            renderable: self.into(),
            size_horiz: FlexSize::Wrap,
            size_vert: FlexSize::Wrap,
        }
    }
}

#[derive(Eq, PartialEq)]
pub enum FlexSize {
    Wrap,
    Grow
}

#[derive(Eq, PartialEq)]
pub enum FlexDir {
    LeftRight,
    UpDown
}

impl From<Flex> for Renderable {
    fn from(value: Flex) -> Self {
        Renderable::Flex(value)
    }
}