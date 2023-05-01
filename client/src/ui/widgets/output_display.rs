use std::fmt::{Debug, Formatter};
use std::iter;
use std::path::Path;
use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;
use tui::style::Color;
use tui::widgets::canvas::Line;
use unicode_segmentation::UnicodeSegmentation;
use crate::ui::widgets::{Cell, Dir, Flow, IntoCell, Renderable, Size, Text};

#[derive(Default)]
pub struct OutputDisplay {
    pub lines: Vec<OutputLine>,
    pub wrap: bool
}
impl OutputDisplay {
    pub fn render<B>(self, rect: Rect, frame: &mut Frame<B>)
        where B: Backend,
    {
        let mut lines: Vec<Vec<LinePart>> = self.lines.into_iter()
            .flat_map(|line| {
                if self.wrap {
                    let mut lines: Vec<Vec<LinePart>> = Vec::new();
                    line.parts.into_iter()
                        .flat_map(|part| {
                            let LinePart { text, color } = part;
                            let mut whitespace_split: Vec<Vec<&str>> = Vec::new();
                            let mut last_whitespace = false;
                            UnicodeSegmentation::graphemes(text.as_str(), true)
                                .for_each(|grapheme| {
                                    // Separate each word into their own vector. Each whitspace-element should end up
                                    // in a vector containing only that element
                                    if whitespace_split.is_empty() || grapheme.trim().len() == 0 || last_whitespace {
                                        whitespace_split.push(Vec::new());
                                    }
                                    last_whitespace = grapheme.trim().len() == 0;
                                    whitespace_split.last_mut().unwrap().push(grapheme);
                                });

                            whitespace_split.into_iter()
                                .map(|graphemes| {
                                    graphemes.concat()
                                })
                                .map(|word| {
                                    LinePart {
                                        text: word.to_string(),
                                        color: color.clone()
                                    }
                                }).collect::<Vec<LinePart>>()
                        })
                        .for_each(|part| {
                            let required_width: usize = lines.last().iter()
                                .flat_map(|vec| vec.iter())
                                .map(|part| part.text.len()).sum::<usize>() + part.text.len();
                            if lines.is_empty() || required_width > rect.width.into() {
                                // Never start wrapped lines with words that are solely whitespace
                                if part.text.chars().any(|char| !char.is_whitespace()) || lines.is_empty() {
                                    lines.push(
                                        line.prefix.iter()
                                            .map(|prefix| prefix.clone())
                                            .chain(if !lines.is_empty() {
                                                Some(LinePart {
                                                    text: String::from("\u{21AA}"),
                                                    color: Color::Rgb(120, 120, 120).into()
                                                })
                                            } else {
                                                None
                                            }.into_iter())
                                            .chain(iter::once(part))
                                            .collect()
                                    );
                                }
                            } else {
                                lines.last_mut().unwrap().push(part);
                            }
                        });

                    lines
                } else {
                    vec![
                        line.prefix.into_iter()
                            .chain(line.parts.into_iter())
                            .collect()
                    ]
                }
            }).collect();

        // Even though the order of lines is from top to bottom, the display is anchored to the line at the bottom.
        // So we only take the amount of lines that we can fit into view.
        if lines.len() > rect.height.into() {
            lines = lines.as_slice()[lines.len().saturating_sub(rect.height.into())..].to_vec();
        }

        Flow {
            cells: lines
                .into_iter()
                .map(|parts| {
                    Cell {
                        element: Flow {
                            direction: Dir::LeftRight,
                            cells: parts.into_iter()
                                .map(|part| {
                                    Cell {
                                        element: Text {
                                            text: part.text,
                                            fg: part.color,
                                            ..Default::default()
                                        }.into_el(),
                                        ..Default::default()
                                    }
                                }).collect(),
                            ..Default::default()
                        }.into_el(),
                        ..Default::default()
                    }
                })
                .collect(),
            direction: Dir::UpDown,
            ..Default::default()
        }.render(rect, frame);
    }

    pub fn measure(&self) -> Size {
        Size {
            width: 0,
            height: 0
        }
    }
}

impl Debug for OutputDisplay {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("OutputDisplay")?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct OutputLine {
    pub prefix: Vec<LinePart>,
    pub parts: Vec<LinePart>
}

#[derive(Clone)]
pub struct LinePart {
    pub text: String,
    pub color: Option<Color>
}

impl From<OutputDisplay> for Renderable {
    fn from(value: OutputDisplay) -> Self {
        Renderable::OutputDisplay(value)
    }
}
