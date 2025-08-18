use crate::system_state::SystemState;
use crate::ui::theming::ATTR_COLOR_FOCUSED_ELEMENT;
use ratatui::style::Color;
use std::sync::{Arc, RwLock};
use ui::component::{Align, Cell, Component, SimpleList, Text};
use ui::{FrameContext, RenderArgs, UIResult};

pub struct SelectProfileScreen {
    pub system_state: Arc<RwLock<SystemState>>,
}

impl Component for SelectProfileScreen {
    type Output = ();

    fn render(
        self,
        context: &mut FrameContext,
    ) -> UIResult<Self::Output> {
        let max_width = context.size().width / 2;
        let max_height = context.size().height / 3;

        let focused_color = context.req_attr::<Color>(ATTR_COLOR_FOCUSED_ELEMENT)?.clone();

        let profile_ids: Vec<String> = {
            let system_state = self.system_state.read().unwrap();
            system_state.config.profiles
                .iter().map(|profile| profile.id.clone()).collect()
        };
        let list_output = context.render_component(
            RenderArgs::new(
                Cell::new(
                    Cell::new(SimpleList::new(
                        "select-profile-list",
                        &profile_ids,
                        |profile_id, _| Ok(Cell::new(Text::new(profile_id)).align(Align::Center)),
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
            let mut system_state = self.system_state.write().unwrap();
            let profile_id = system_state.config.profiles[selection.selected_index].id.clone();
            system_state.select_profile(&profile_id);
        }

        Ok(())
    }
}
