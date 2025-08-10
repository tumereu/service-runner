use ratatui::style::Color;
use ratatui::Frame;
use ui::{render, Canvas, RenderContext};
use ui::component::{Component, Measurement, Text};
use crate::system_state::SystemState;
use crate::ui::widgets::{render_root, Align, Cell, IntoCell, List};
use crate::ui::CurrentScreen;

#[derive(Default)]
pub struct SelectProfileState {
    selected_idx: usize,
}

pub struct SelectProfileScreen {
}
impl Component for SelectProfileScreen {
    type State = SelectProfileState;
    type Output = ();

    fn measure(&self, canvas: &Canvas, ctx: RenderContext<SelectProfileState>) -> Measurement {
        Default::default()
    }

    fn render(&self, canvas: &Canvas, ctx: RenderContext<SelectProfileState>) -> Self::Output {
        render!(canvas, {
            key = "test",
            component = Text {
                text: "Hello world! ".repeat(100),
                fg: Some(Color::Cyan),
                ..Default::default()
            },
            pos = (4, 8),
        });
    }
}

pub fn render_profile_select(frame: &mut Frame, state: &SystemState)
{
    let selected_idx = match &state.ui.screen {
        CurrentScreen::ProfileSelect { selected_idx } => selected_idx,
        any => panic!("Invalid UI state in render_profile_select: {any:?}"),
    };

    // TODO theme?
    let active_border_color = Color::Rgb(180, 180, 0);

    render_root(
        Cell {
            align_vert: Align::Center,
            align_horiz: Align::Center,
            element: Cell {
                align_vert: Align::Center,
                align_horiz: Align::Stretch,
                border: Some((active_border_color, "Select profile".into())),
                min_width: 16,
                fill: false,
                element: List {
                    items: List::simple_items(
                        state
                            .config
                            .profiles
                            .iter()
                            .map(|prof| prof.id.clone())
                            .collect(),
                        Align::Center
                    ),
                    selection: *selected_idx,
                }.into_el(),
                ..Default::default()
            }.into_el(),
            ..Default::default()
        },
        frame,
    );
}
