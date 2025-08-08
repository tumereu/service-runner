use std::fmt::{Debug, Formatter};
use std::iter;

use itertools::Itertools;
use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::Color;
use tui::widgets::Clear;
use tui::Frame;
use unicode_segmentation::UnicodeSegmentation;

use crate::ui::widgets::{Cell, Dir, Flow, IntoCell, Renderable, Size, Text};

#[derive(Default)]
pub struct OutputDisplay {
    pub lines: Vec<OutputLine>,
    pub pos_horiz: Option<u64>,
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
                    // Special case handling: empty lines/line breaks in the output. These should result in an empty
                    // line and no wrapping
                    if line.parts.iter().all(|part| part.text.chars().all(|char| char.is_whitespace())) {
                        lines.push(
                            line.prefix.to_vec()
                        );
                    } else {
                        line.parts.into_iter()
                            .flat_map(|part| {
                                let LinePart { text, color } = part;
                                let mut whitespace_split: Vec<Vec<&str>> = Vec::new();
                                let mut last_whitespace = false;
                                UnicodeSegmentation::graphemes(text.as_str(), true)
                                    .for_each(|grapheme| {
                                        // Separate each word into their own vector. Each whitspace-element should end up
                                        // in a vector containing only that element
                                        if whitespace_split.is_empty() || grapheme.trim().is_empty() || last_whitespace {
                                            whitespace_split.push(Vec::new());
                                        }
                                        last_whitespace = grapheme.trim().is_empty();
                                        whitespace_split.last_mut().unwrap().push(grapheme);
                                    });

                                whitespace_split.into_iter()
                                    .map(|graphemes| {
                                        graphemes.concat()
                                    })
                                    .map(|word| {
                                        LinePart {
                                            text: word.chars().map(|char| {
                                                // Replace all whitespace characters with standard spaces to prevent
                                                // weird rendering issues
                                                if char.is_whitespace() {
                                                    // TODO replace tabs with multiple spaces?
                                                    ' '
                                                } else {
                                                    char
                                                }
                                            }).collect(),
                                            color
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
                                            line.prefix.iter().cloned()
                                                .chain(if !lines.is_empty() {
                                                    Some(LinePart {
                                                        text: String::from("\u{21AA}"),
                                                        color: Color::Rgb(120, 120, 120).into()
                                                    })
                                                } else {
                                                    None
                                                })
                                                .chain(iter::once(part))
                                                .collect()
                                        );
                                    }
                                } else {
                                    lines.last_mut().unwrap().push(part);
                                }
                            });
                    }

                    lines
                } else {
                    let mut remaining_to_drop = self.pos_horiz.unwrap_or(0) as usize;
                    vec![
                        line.prefix.into_iter()
                            .chain(line.parts.into_iter())
                            .map(|part| {
                                let new_part = LinePart {
                                    text: part.text.graphemes(true)
                                        .enumerate()
                                        .filter(|(index, _grapheme)| index >= &remaining_to_drop)
                                        .map(|(_, grapheme)| {
                                            // Replace all whitespace with space.
                                            if grapheme.trim().is_empty() {
                                                // TODO replace tabs with multiple spaces?
                                                &" "
                                            } else {
                                                grapheme
                                            }
                                        })
                                        .collect::<Vec<&str>>()
                                        .concat(),
                                    color: part.color
                                };
                                remaining_to_drop = remaining_to_drop.saturating_sub(part.text.len());
                                new_part
                            })
                            .collect()
                    ]
                }
            }).collect();

        // Even though the order of lines is from top to bottom, the display is anchored to the line at the bottom.
        // So we only take the amount of lines that we can fit into view.
        if lines.len() > rect.height.into() {
            lines = lines.as_slice()[lines.len().saturating_sub(rect.height.into())..].to_vec();
        }

        // Clear the area before rendering
        frame.render_widget(Clear, rect);

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
