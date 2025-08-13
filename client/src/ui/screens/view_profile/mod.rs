mod output_display;
mod output_pane;
mod service_list;

use crate::system_state::SystemState;
use crate::ui::actions::ActionStore;
use crate::ui::theming::{ATTR_COLOR_FOCUSED_ELEMENT, ATTR_COLOR_UNFOCUSED_ELEMENT};
use ratatui::prelude::Color;
use ui::component::{Align, Cell, Component, StatefulComponent, WithMeasurement};
use ui::{FrameContext, RenderArgs, UIError, UIResult};
use ui::input::{KeyMatcher, KeyMatcherQueryable};

pub struct ViewProfileScreen<'a> {
    pub state: &'a SystemState,
    pub actions: &'a ActionStore,
}
impl<'a> StatefulComponent for ViewProfileScreen<'a> {
    type State = ViewProfileScreenState;
    type Output = ();

    fn state_id(&self) -> &str {
        "view-profile-screen"
    }

    fn render(
        &self,
        context: &mut FrameContext,
        state: &mut Self::State,
    ) -> UIResult<Self::Output> {
        let service_list_component = service_list::ServiceList { state: self.state };
        let self_size = context.size();
        let list_width = context.measure_component(&service_list_component)?.width + 2;
        let list_height = self_size.height / 2 + 2;

        let focused_color = context
            .req_attr::<Color>(ATTR_COLOR_FOCUSED_ELEMENT)?
            .clone();
        let unfocused_color = context
            .req_attr::<Color>(ATTR_COLOR_UNFOCUSED_ELEMENT)?
            .clone();

        // TODO move into settings file and attributes
        if context.signals().is_key_pressed(vec![KeyMatcher::char('w')]) {
            state.wrap_output = !state.wrap_output;
        }

        let profile_name = &self
            .state
            .current_profile
            .as_ref()
            .ok_or(UIError::IllegalState {
                msg: "No profile selected".to_string(),
            })?
            .definition
            .id;

        context.render_component(
            RenderArgs::new(
                &Cell::new(
                    service_list::ServiceList { state: self.state }
                        .with_measurement(0u16, 0u16),
                )
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
            .size(list_width, list_height)
            .pos(0, 0),
        )?;

        context.render_component(
            RenderArgs::new(
                &Cell::new(
                    output_pane::OutputPane {
                        wrap_output: state.wrap_output,
                        // TODO move as inner state?
                        pos_horiz: None,
                        pos_vert: None,
                        state: self.state,
                    }
                    .with_measurement(0u16, 0u16),
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
            .size(self_size.width - list_width, self_size.height)
            .pos(list_width, 0),
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
