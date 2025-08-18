mod output_display;
mod output_pane;
mod service_list;

use crate::system_state::SystemState;
use crate::ui::inputs::{ATTR_KEY_FOCUS_NEXT, ATTR_KEY_FOCUS_PREV, ATTR_KEY_TOGGLE_WRAP};
use crate::ui::theming::{ATTR_COLOR_FOCUSED_ELEMENT, ATTR_COLOR_UNFOCUSED_ELEMENT};
use ratatui::layout::Size;
use ratatui::prelude::Color;
use std::sync::{Arc, RwLock};
use ui::component::{
    Align, Cell, Component, StatefulComponent, WithMeasurement, WithZeroMeasurement,
};
use ui::input::KeyMatcherQueryable;
use ui::{FrameContext, RenderArgs, SignalHandling, UIError, UIResult};

pub struct ViewProfileScreen {
    pub system_state: Arc<RwLock<SystemState>>,
}
impl StatefulComponent for ViewProfileScreen {
    type State = ViewProfileScreenState;
    type Output = ();

    fn state_id(&self) -> &str {
        "view-profile-screen"
    }

    fn render(
        self,
        context: &mut FrameContext,
        state: &mut Self::State,
    ) -> UIResult<Self::Output> {
        if context
            .signals()
            .is_key_pressed(context.req_attr(ATTR_KEY_TOGGLE_WRAP)?)
        {
            state.wrap_output = !state.wrap_output;
        }
        if context
            .signals()
            .is_key_pressed(context.req_attr(ATTR_KEY_FOCUS_NEXT)?)
        {
            state.focused_pane = match state.focused_pane {
                FocusedPane::ServiceList => FocusedPane::OutputArea,
                FocusedPane::OutputArea => FocusedPane::ServiceList,
            };
        } else if context
            .signals()
            .is_key_pressed(context.req_attr(ATTR_KEY_FOCUS_PREV)?)
        {
            state.focused_pane = match state.focused_pane {
                FocusedPane::ServiceList => FocusedPane::OutputArea,
                FocusedPane::OutputArea => FocusedPane::ServiceList,
            };
        }

        let focused_color = context
            .req_attr::<Color>(ATTR_COLOR_FOCUSED_ELEMENT)?
            .clone();
        let unfocused_color = context
            .req_attr::<Color>(ATTR_COLOR_UNFOCUSED_ELEMENT)?
            .clone();
        let self_size = context.size();

        let (service_list_component, list_size) = {
            let system_state = self.system_state.read().unwrap();
            let list_component = service_list::ServiceList {
                system_state: self.system_state.clone(),
                show_selection: state.focused_pane == FocusedPane::ServiceList,
            };
            let list_width = context.measure_component(&list_component)?.width + 2;
            let list_height = self_size.height / 2 + 2;

            let profile_name = &system_state
                .current_profile
                .as_ref()
                .ok_or(UIError::IllegalState {
                    msg: "No profile selected".to_string(),
                })?
                .definition
                .id;

            let render_args = RenderArgs::new(
                Cell::new(list_component.with_zero_measurement())
                    .border(
                        if state.focused_pane == FocusedPane::ServiceList {
                            focused_color
                        } else {
                            unfocused_color
                        },
                        profile_name,
                    )
                    .align(Align::Stretch),
            )
                .signals(if state.focused_pane == FocusedPane::ServiceList {
                    SignalHandling::Forward
                } else {
                    SignalHandling::Block
                })
                .size(list_width, list_height)
                .pos(0, 0);

            (render_args, Size { width: list_width, height: list_height })
        };

        context.render_component(service_list_component)?;

        context.render_component(
            RenderArgs::new(
                Cell::new(
                    output_pane::OutputPane {
                        wrap_output: state.wrap_output,
                        system_state: self.system_state.clone(),
                    }.with_zero_measurement(),
                )
                    .border(
                        if state.focused_pane == FocusedPane::OutputArea {
                            focused_color
                        } else {
                            unfocused_color
                        },
                        if state.wrap_output {
                            "Wrap: Y"
                        } else {
                            "Wrap: N"
                        },
                    )
                    .align(Align::Stretch),
            )
                .signals(if state.focused_pane == FocusedPane::OutputArea {
                    SignalHandling::Forward
                } else {
                    SignalHandling::Block
                })
                .size(self_size.width - list_size.width, self_size.height)
                .pos(list_size.width, 0),
        )?;

        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct ViewProfileScreenState {
    focused_pane: FocusedPane,
    wrap_output: bool,
}
#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
pub enum FocusedPane {
    ServiceList,
    OutputArea,
}

impl Default for FocusedPane {
    fn default() -> Self {
        FocusedPane::ServiceList
    }
}
