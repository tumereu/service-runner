use crate::models::{OutputKind, Profile, get_active_outputs};
use crate::system_state::SystemState;
use crate::ui::screens::view_profile::output_display::{LinePart, OutputDisplay, OutputLine};
use ratatui::style::Color;
use std::hash::{DefaultHasher, Hash, Hasher};
use log::debug;
use ui::component::{Component, Dir, Flow, FlowableArgs, Spinner};
use ui::{FrameContext, RenderArgs, UIResult};

pub struct OutputPane<'a> {
    pub wrap_output: bool,
    pub pos_horiz: Option<u64>,
    pub pos_vert: Option<u128>,
    pub state: &'a SystemState,
}
impl OutputPane<'_> {
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

impl<'a> Component for OutputPane<'a> {
    type Output = ();

    fn render(&self, context: &mut FrameContext) -> UIResult<Self::Output> {
        let OutputPane {
            wrap_output,
            pos_horiz,
            pos_vert,
            state,
        } = self;
        let theme = &state.config.settings.theme;
        let profile = state.current_profile.as_ref().unwrap();
        let size = context.size();

        let mut flow = Flow::new().dir(Dir::UpDown).element(
            OutputDisplay {
                wrap: *wrap_output,
                pos_horiz: *pos_horiz,
                lines: state
                    .output_store
                    .query_lines_to(
                        size.height as usize,
                        *pos_vert,
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
                                        // TODO move into theme
                                        OutputKind::System => Color::Rgb(0, 180, 0),
                                        OutputKind::ExtProcess => Color::Rgb(0, 120, 220),
                                    }
                                    .into(),
                                },
                                LinePart {
                                    text: format!("{name}/"),
                                    color: theme.service_colors
                                        [color_idx % theme.service_colors.len()]
                                    .into(),
                                },
                                LinePart {
                                    text: format!(
                                        "{name} | ",
                                        name = Self::force_len(&key.source_name, 5)
                                    ),
                                    color: Some(
                                        theme.source_colors[Self::hash_name(&key.source_name)
                                            & theme.source_colors.len()],
                                    ),
                                },
                            ],
                            parts: vec![LinePart {
                                text: line.value.clone(),
                                color: None,
                            }],
                        }
                    })
                    .collect(),
            },
            FlowableArgs { fill: true },
        );

        if pos_vert.is_none() {
            flow = flow.element(Spinner::new(true), FlowableArgs { fill: false });
        }

        context.render_component(RenderArgs::new(&flow))?;

        Ok(())
    }
}
