use crate::models::{OutputKind, Profile, get_active_outputs};
use crate::system_state::SystemState;
use crate::ui::screens::view_profile::output_display::{LinePart, OutputDisplay, OutputLine};
use ratatui::style::Color;
use std::hash::{DefaultHasher, Hash, Hasher};
use log::debug;
use ratatui::layout::Size;
use ui::component::{Component, Dir, Flow, FlowableArgs, MeasurableComponent, Spinner, StatefulComponent};
use ui::{FrameContext, RenderArgs, UIResult};

pub struct OutputPane<'a> {
    pub wrap_output: bool,
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

#[derive(Default)]
pub struct OutputPaneState {
    pub pos_horiz: Option<u64>,
    pub pos_vert: Option<u128>,
}

impl<'a> StatefulComponent for OutputPane<'a> {
    type Output = ();
    type State = OutputPaneState;

    fn state_id(&self) -> &str {
        "view-profile-output-pane"
    }

    fn render(self, context: &mut FrameContext, state: &mut Self::State) -> UIResult<Self::Output> {
        let OutputPane {
            wrap_output,
            state: system_state,
        } = self;
        let theme = &system_state.config.settings.theme;
        let profile = system_state.current_profile.as_ref().unwrap();
        let size = context.size();

        let mut flow = Flow::new().dir(Dir::UpDown).element(
            OutputDisplay {
                wrap: wrap_output,
                pos_horiz: state.pos_horiz,
                lines: system_state
                    .output_store
                    .query_lines_to(
                        size.height as usize,
                        state.pos_vert,
                        &get_active_outputs(&system_state.output_store, system_state),
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
                                        name = key.source_name
                                    ),
                                    color: Some(
                                        theme.source_colors[Self::hash_name(&key.source_name)
                                            % theme.source_colors.len()],
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

        // TODO maybe we can display this in a manner that doesn't look like its loading something?
        if state.pos_vert.is_none() {
            flow = flow.element(Spinner::new(true), FlowableArgs { fill: false });
        }

        context.render_component(RenderArgs::new(flow))?;

        Ok(())
    }
}