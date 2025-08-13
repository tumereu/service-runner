mod service_list;

use crate::system_state::SystemState;
use ratatui::prelude::Color;
use ui::component::{Align, Cell, Component, Text, WithMeasurement};
use ui::{FrameContext, RenderArgs, UIError, UIResult};
use crate::ui::actions::ActionStore;

pub struct ViewProfileScreen<'a> {
    pub state: &'a SystemState,
    pub actions: &'a ActionStore,
}
impl<'a> Component for ViewProfileScreen<'a> {
    type Output = ();

    fn render(&self, context: &mut FrameContext) -> UIResult<Self::Output> {
        let service_list_component = service_list::ServiceList {
            state: self.state,
        };
        let list_width = context.measure_component(&service_list_component)?.width;
        let list_height = context.size().height / 2;

        let profile_name = &self.state.current_profile.as_ref().ok_or(
            UIError::IllegalState {
                msg: "No profile selected".to_string(),
            }
        )?.definition.id;

        context.render_component(
            RenderArgs::new(
                &Cell::new(
                    service_list::ServiceList {
                        state: self.state,
                    }.with_measurement(list_width, list_height),
                )
                .border(Color::Yellow, profile_name)
                .align(Align::Start),
            )
            .size(list_width + 2, list_height + 2)
            .pos(0, 0)
        )?;

        Ok(())
    }
}
