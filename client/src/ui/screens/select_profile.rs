use crate::system_state::SystemState;
use ratatui::style::Color;
use ui::component::{Align, Cell, Component, List, StatefulComponent, Text, ATTR_KEY_SELECT};
use ui::{FrameContext, RenderArgs, UIResult};
use ui::input::KeyMatcherQueryable;
use crate::ui::actions::{Action, ActionStore};
use crate::ui::theming::{ATTR_COLOR_FOCUSED_ELEMENT, ATTR_COLOR_UNFOCUSED_ELEMENT};

pub struct SelectProfileScreen<'a> {
    pub state: &'a SystemState,
    pub actions: &'a ActionStore,
}

impl<'a> StatefulComponent for SelectProfileScreen<'a> {
    type State = SelectProfileScreenState;
    type Output = ();

    fn state_id(&self) -> &str {
        "select-profile-screen"
    }

    fn render(
        &self,
        context: &mut FrameContext,
        state: &mut Self::State,
    ) -> UIResult<Self::Output> {
        let max_width = context.size().width / 2;
        let max_height = context.size().height / 3;

        let focused_color = context.req_attr::<Color>(ATTR_COLOR_FOCUSED_ELEMENT)?.clone();
        let unfocused_color = context.req_attr::<Color>(ATTR_COLOR_UNFOCUSED_ELEMENT)?.clone();

        let list_output = context.render_component(
            RenderArgs::new(
                &Cell::new(
                    Cell::new(List::new(
                        "select-profile-list",
                        &self.state.config.profiles,
                        |profile, _| Ok(Cell::new(Text::new(profile.id.clone())).align(Align::Center)),
                    ))
                    .border(
                        if state.focused_pane == FocusedPane::ServiceList {
                            focused_color
                        } else {
                            unfocused_color
                        },
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

        if context.signals().is_key_pressed(context.req_attr(ATTR_KEY_SELECT)?) {
            self.actions.register(Action::SelectProfile(
                self.state.config.profiles[list_output.selected_index].id.clone()
            ));
        }

        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct SelectProfileScreenState {
    focused_pane: FocusedPane,
}
#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
pub enum FocusedPane {
    ServiceList,
    OutputArea
}

impl Default for FocusedPane {
    fn default() -> Self {
        FocusedPane::ServiceList
    }
}
