use crate::system_state::SystemState;
use ratatui::style::Color;
use ui::component::{Align, Cell, Component, List, Text, ATTR_KEY_SELECT};
use ui::{FrameContext, RenderArgs, UIResult};
use ui::input::KeyMatcherQueryable;
use crate::ui::actions::{Action, ActionStore};

pub struct SelectProfileScreen<'a> {
    pub state: &'a SystemState,
    pub actions: &'a ActionStore,
}
impl<'a> Component for SelectProfileScreen<'a> {
    type Output = ();

    fn render(
        &self,
        context: &mut FrameContext,
    ) -> UIResult<Self::Output> {
        let max_width = context.size().width / 2;
        let max_height = context.size().height / 3;

        let list_output = context.render_component(
            RenderArgs::new(
                &Cell::new(
                    Cell::new(List::new(
                        "select-profile-list",
                        &self.state.config.profiles,
                        |profile, _| Cell::new(Text::new(profile.id.clone())).align(Align::Center),
                    ))
                    .border(Color::Yellow, "Select profile")
                    .bg(Color::Reset)
                    .min_width(20)
                    .max_width(max_width)
                    .max_height(max_height),
                )
                .align(Align::Center),
            )
        )?;

        if context.signals().is_key_pressed(context.req_attr(ATTR_KEY_SELECT)?) {
            self.actions.register(Action::SelectProfile(
                self.state.config.profiles[list_output.selected_index].id.clone()
            ));
        }

        Ok(())
    }
}