use crate::ui::Component;
use ratatui::prelude::Size;
use ratatui::style::Color;
use ratatui::widgets::Clear;
use std::iter;
use ui::component::{Dir, Flow, FlowableArgs, MeasurableComponent, Text};
use ui::{FrameContext, RenderArgs, UIResult};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default)]
pub struct OutputDisplay {
    pub lines: Vec<OutputLine>,
    pub pos_horiz: Option<u64>,
    pub wrap: bool
}
impl Component for OutputDisplay {
    type Output = ();

    fn render(self, context: &mut FrameContext) -> UIResult<Self::Output>
    {
        let size = context.size();

        // TODO this uses clones etc and surely could be performance optimized. Should render take ownership of the
        // element?
        let mut lines: Vec<Vec<LinePart>> = self.lines.iter()
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
                        line.parts.iter()
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
                                            color: color.clone(),
                                        }
                                    }).collect::<Vec<LinePart>>()
                            })
                            .for_each(|part| {
                                let required_width: usize = lines.last().iter()
                                    .flat_map(|vec| vec.iter())
                                    .map(|part| part.text.len()).sum::<usize>() + part.text.len();
                                if lines.is_empty() || required_width > size.width.into() {
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
                        line.prefix.iter()
                            .chain(line.parts.iter())
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
        if lines.len() > size.height.into() {
            lines = lines.as_slice()[lines.len().saturating_sub(size.height.into())..].to_vec();
        }

        // Clear the area before rendering
        context.render_widget(Clear, (0, 0).into(), size);

        let mut flow = Flow::new().dir(Dir::UpDown);

        for line in lines {
            let mut inner_flow = Flow::new().dir(Dir::LeftRight);
            for part in line {
                inner_flow = inner_flow.element(
                    Text::new(part.text).fg(part.color),
                    FlowableArgs { fill: false },
                )
            }
            flow = flow.element(inner_flow, FlowableArgs { fill: false });
        }

        context.render_component(RenderArgs::new(flow))?;

        Ok(())
    }
}
impl MeasurableComponent for OutputDisplay {
    fn measure(&self, _context: &FrameContext) -> UIResult<Size> {
        Ok(Size { width: 0, height: 0})
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
