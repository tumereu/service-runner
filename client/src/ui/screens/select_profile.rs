use crate::system_state::SystemState;
use crate::ui::CurrentScreen;
use ratatui::Frame;
use ratatui::style::Color;
use ui::component::{Align, Cell, Component, Text};
use ui::{FrameContext, RenderArgs};

#[derive(Default)]
pub struct SelectProfileState {
    selected_idx: usize,
}

pub struct SelectProfileScreen {}
impl Component for SelectProfileScreen {
    type State = SelectProfileState;
    type Output = ();

    fn render(&self, context: &FrameContext, _state: &mut Self::State) -> Self::Output {
        let text = context.on_signal(|signal: String| signal.to_owned());

        context.render_component(&RenderArgs::new(
            Cell::containing(
                Cell::containing(Text {
                    text: text.unwrap_or("Hello cell".into()),
                    fg: Some(Color::Cyan),
                    ..Default::default()
                })
                .min_width(16)
                .min_height(12)
                .border(Color::Yellow, "Select profile")
                .bg(Color::Reset)
            )
            .align(Align::Center),
        ).key("root"));
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
