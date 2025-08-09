use std::hash::{DefaultHasher, Hash, Hasher};
use std::iter;
use ratatui::style::Color;
use crate::models::{get_active_outputs, OutputKind, Profile};
use crate::system_state::SystemState;
use crate::ui::widgets::{Align, Cell, Dir, Flow, IntoCell, LinePart, OutputDisplay, OutputLine, Spinner};

pub struct OutputPane<'a> {
    pub height: usize,
    pub wrap_output: bool,
    pub pos_horiz: Option<u64>,
    pub pos_vert: Option<u128>,
    pub profile: &'a Profile,
    pub state: &'a SystemState,
}

impl OutputPane<'_> {
    pub fn render(self) -> Flow {
        let OutputPane {
            height,
            wrap_output,
            pos_horiz,
            pos_vert,
            profile,
            state
        } = self;
        let theme = &state.config.settings.theme;

        Flow {
            direction: Dir::UpDown,
            cells: iter::once(Cell {
                align_horiz: Align::Stretch,
                align_vert: Align::Stretch,
                fill: true,
                element: OutputDisplay {
                    wrap: wrap_output,
                    pos_horiz,
                    lines: state
                        .output_store
                        .query_lines_to(
                            height,
                            pos_vert,
                            &get_active_outputs(&state.output_store, state),
                        )
                        .into_iter()
                        .map(|(key, line)| {
                            let color_idx = key
                                .service_id
                                .clone()
                                .and_then(|service_id| {
                                    profile
                                        .services
                                        .iter()
                                        .enumerate()
                                        .find(|(_, service)| service.definition.id == service_id)
                                        .map(|(idx, _)| idx)
                                })
                                .unwrap_or(profile.services.len());
                            let name = key
                                .service_id
                                .clone()
                                .unwrap_or(profile.definition.id.clone());

                            OutputLine {
                                prefix: vec![
                                    LinePart {
                                        text: match key.kind {
                                            OutputKind::System => "i/",
                                            OutputKind::ExtProcess => "c/",
                                        }
                                            .to_string(),
                                        color: match key.kind {
                                            OutputKind::System => Color::Rgb(0, 180, 0),
                                            OutputKind::ExtProcess => Color::Rgb(0, 120, 220),
                                        }
                                            .into(),
                                    },
                                    LinePart {
                                        text: format!("{name}/"),
                                        color: theme.service_colors[color_idx % theme.service_colors.len()]
                                            .into(),
                                    },
                                    LinePart {
                                        text: format!(
                                            "{name} | ",
                                            name = Self::force_len(&key.source_name, 5)
                                        ),
                                        color: Some(
                                            theme.source_colors[
                                                Self::hash_name(&key.source_name) & theme.source_colors.len()
                                                ]
                                        )
                                    },
                                ],
                                parts: vec![LinePart {
                                    text: line.value.clone(),
                                    color: None,
                                }],
                            }
                        })
                        .collect(),
                }
                    .into_el(),
                ..Default::default()
            })
                .chain(if pos_vert.is_none() {
                    Some(Cell {
                        align_horiz: Align::Stretch,
                        element: Spinner {
                            active: true,
                            ..Default::default()
                        }
                            .into_el(),
                        ..Default::default()
                    })
                } else {
                    None
                })
                .collect(),
            ..Default::default()
        }
    }

    fn hash_name(name: &str) -> usize {
        // Hash the name to obtain a color for it
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        hasher.finish() as usize
    }

    fn force_len(text: &str, len: usize) -> String {
        let actual_len = text.chars().count();

        if actual_len == len {
            text.to_string()
        } else if actual_len > len {
            text.chars().take(len).collect()
        } else {
            let padding = " ".repeat(len - actual_len);
            format!("{}{}", text, padding)
        }
    }
}