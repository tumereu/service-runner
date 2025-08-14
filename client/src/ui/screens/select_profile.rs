use crate::system_state::SystemState;
use ratatui::style::Color;
use ui::component::{Align, Cell, Component, SimpleList, StatefulComponent, Text, ATTR_KEY_SELECT};
use ui::{FrameContext, RenderArgs, UIResult};
use ui::input::KeyMatcherQueryable;
use crate::ui::actions::{Action, ActionStore};
use crate::ui::theming::{ATTR_COLOR_FOCUSED_ELEMENT, ATTR_COLOR_UNFOCUSED_ELEMENT};

pub struct SelectProfileScreen<'a> {
    pub state: &'a SystemState,
    pub actions: &'a ActionStore,
}

impl<'a> Component for SelectProfileScreen<'a> {
    type Output = ();

    fn render(
        self,
        context: &mut FrameContext,
    ) -> UIResult<Self::Output> {
        let max_width = context.size().width / 2;
        let max_height = context.size().height / 3;

        let focused_color = context.req_attr::<Color>(ATTR_COLOR_FOCUSED_ELEMENT)?.clone();

        let list_output = context.render_component(
            RenderArgs::new(
                Cell::new(
                    Cell::new(SimpleList::new(
                        "select-profile-list",
                        &self.state.config.profiles,
                        |profile, _| Ok(Cell::new(Text::new(profile.id.clone())).align(Align::Center)),
                    ))
                    .border(
                        focused_color,
                        "Select profile"
                    )
                    .bg(Color::Reset)
                    .min_width(20)
                    .max_width(max_width)
                    .max_height(max_height),
                )
                .align(Align::Center),
            )
        )?;

        if let Some(selection) = list_output {
            self.actions.register(Action::SelectProfile(
                self.state.config.profiles[selection.selected_index].id.clone()
            ));
        }

        Ok(())
    }
}
