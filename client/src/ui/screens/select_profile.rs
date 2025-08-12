use crate::config::ProfileDefinition;
use crate::system_state::SystemState;
use crate::ui::CurrentScreen;
use ratatui::Frame;
use ratatui::style::Color;
use ui::component::{Align, Cell, Component, List, Text, WithMeasurement};
use ui::{FrameContext, RenderArgs, UIResult};

#[derive(Default)]
pub struct SelectProfileState {
    selected_idx: usize,
}

pub struct SelectProfileScreen<'a> {
    pub profiles: &'a Vec<ProfileDefinition>,
}
impl<'a> Component for SelectProfileScreen<'a> {
    type Output = ();

    fn render(
        &self,
        context: &mut FrameContext,
    ) -> UIResult<Self::Output> {
        let max_width = context.size().width / 2;
        let max_height = context.size().height / 3;

        let mock_profiles = (0..100)
            .into_iter()
            .map(|i| ProfileDefinition {
                id: i.to_string(),
                ..self.profiles[0].clone()
            })
            .collect::<Vec<_>>();

        context.render_component(
            RenderArgs::new(
                &Cell::new(
                    Cell::new(List::new(
                        "select-profile-list",
                        &mock_profiles,
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
        )
    }
}

pub fn render_profile_select(frame: &mut Frame, state: &SystemState) {
    let selected_idx = match &state.ui.screen {
        CurrentScreen::ProfileSelect { selected_idx } => selected_idx,
        any => panic!("Invalid UI state in render_profile_select: {any:?}"),
    };

    // TODO theme?
    let active_border_color = Color::Rgb(180, 180, 0);

    /*
    render_root(
        Cell {
            align_vert: Align::Center,
            align_horiz: Align::Center,
            content: Cell {
                align_vert: Align::Center,
                align_horiz: Align::Stretch,
                border: Some((active_border_color, "Select profile".into())),
                min_width: 16,
                fill: false,
                content: List {
                    items: List::simple_items(
                        state
                            .config
                            .profiles
                            .iter()
                            .map(|prof| prof.id.clone())
                            .collect(),
                        Align::Center,
                    ),
                    selection: *selected_idx,
                }
                .into_el(),
                ..Default::default()
            }
            .into_el(),
            ..Default::default()
        },
        frame,
    );

     */
}
