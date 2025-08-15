use crate::models::{OutputKind, Profile, get_active_outputs};
use crate::system_state::SystemState;
use crate::ui::screens::view_profile::output_display::{LinePart, OutputDisplay, OutputLine};
use log::debug;
use ratatui::layout::Size;
use ratatui::style::Color;
use std::cmp::max;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::{Arc, Mutex, RwLock};
use ui::component::{ATTR_KEY_NAV_DOWN, ATTR_KEY_NAV_DOWN_LARGE, ATTR_KEY_NAV_UP, ATTR_KEY_NAV_UP_LARGE, Component, Dir, Flow, FlowableArgs, MeasurableComponent, Spinner, StatefulComponent, ATTR_KEY_NAV_LEFT, ATTR_KEY_NAV_LEFT_LARGE, ATTR_KEY_NAV_RIGHT, ATTR_KEY_NAV_RIGHT_LARGE};
use ui::input::KeyMatcherQueryable;
use ui::{FrameContext, RenderArgs, UIResult};

pub struct OutputPane {
    pub wrap_output: bool,
    pub system_state: Arc<RwLock<SystemState>>
}
impl OutputPane {
    fn hash_name(name: &str) -> usize {
        // Hash the name to obtain a color for it
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        hasher.finish() as usize
    }

    fn process_inputs(
        &self,
        context: &mut FrameContext,
        system: &SystemState,
        state: &mut OutputPaneState,
    ) -> UIResult<()> {
        let nav_down = if context
            .signals()
            .is_key_pressed(context.req_attr(ATTR_KEY_NAV_DOWN)?)
        {
            Some(1)
        } else if context
            .signals()
            .is_key_pressed(context.req_attr(ATTR_KEY_NAV_DOWN_LARGE)?)
        {
            Some(context.size().height as usize / 2)
        } else {
            None
        };
        let nav_up = if context
            .signals()
            .is_key_pressed(context.req_attr(ATTR_KEY_NAV_UP)?)
        {
            Some(1)
        } else if context
            .signals()
            .is_key_pressed(context.req_attr(ATTR_KEY_NAV_UP_LARGE)?)
        {
            Some(context.size().height as usize / 2)
        } else {
            None
        };
        if context.signals().is_key_pressed(context.req_attr(ATTR_KEY_NAV_LEFT_LARGE)?) {
            state.pos_horiz = state.pos_horiz.saturating_sub(context.size().width as u64 / 2);
        } else if context.signals().is_key_pressed(context.req_attr(ATTR_KEY_NAV_LEFT)?) {
            state.pos_horiz = state.pos_horiz.saturating_sub(1);
        } else if context.signals().is_key_pressed(context.req_attr(ATTR_KEY_NAV_RIGHT_LARGE)?) {
            state.pos_horiz = state.pos_horiz.saturating_add(context.size().width as u64 / 2);
        } else if context.signals().is_key_pressed(context.req_attr(ATTR_KEY_NAV_RIGHT)?) {
            state.pos_horiz = state.pos_horiz.saturating_add(1);
        }

        if let Some(amount) = nav_up {
            let active_outputs = get_active_outputs(&system.output_store, &system);
            // Prevent users from scrolling past the first line of output
            // TODO this actually prevents viewing the first lines if wrapping is on, as it doesn't account for that.
            let min_index = system
                .output_store
                .query_lines_from(
                    context.size().height as usize,
                    None,
                    &active_outputs,
                )
                .last()
                .unwrap()
                .1
                .index;

            state.pos_vert = system
                .output_store
                .query_lines_to(
                    amount + if state.pos_vert.is_none() { 0 } else { 1 },
                    state.pos_vert,
                    &active_outputs,
                )
                .first()
                .map(|(_, line)| max(line.index, min_index));
        } else if let Some(amount) = nav_down {
            state.pos_vert = state.pos_vert.and_then(|pos| {
                let lines = system.output_store.query_lines_from(
                    amount + 1,
                    Some(pos),
                    &get_active_outputs(&system.output_store, &system),
                );
                if lines.len() == amount + 1 {
                    lines.last().map(|(_, line)| line.index)
                } else {
                    None
                }
            });
        }

        // TODO check available space and set to auto-scroll if there's less lines than height (accountign for wrapping)

        Ok(())
    }
}

#[derive(Default)]
pub struct OutputPaneState {
    pub pos_horiz: u64,
    pub pos_vert: Option<u128>,
}

impl StatefulComponent for OutputPane {
    type Output = ();
    type State = OutputPaneState;

    fn state_id(&self) -> &str {
        "view-profile-output-pane"
    }

    fn render(self, context: &mut FrameContext, state: &mut Self::State) -> UIResult<Self::Output> {
        let system_state = self.system_state.read().unwrap();
        self.process_inputs(context, &system_state, state)?;

        let theme = &system_state.config.settings.theme;
        let profile = system_state.current_profile.as_ref().unwrap();
        let size = context.size();


        let mut flow = Flow::new().dir(Dir::UpDown).element(
            OutputDisplay {
                wrap: self.wrap_output,
                pos_horiz: Some(state.pos_horiz),
                lines: system_state
                    .output_store
                    .query_lines_to(
                        size.height as usize,
                        state.pos_vert,
                        &get_active_outputs(&system_state.output_store, &system_state),
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
                            .service_id.as_ref().map(|id| id.inner().to_owned())
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
                                    text: format!("{name} | ", name = key.source_name),
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
