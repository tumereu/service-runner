use tui::backend::Backend;
use tui::Frame;
use tui::style::Color;

use crate::system_state::SystemState;
use crate::ui::CurrentScreen;
use crate::ui::widgets::{Align, Cell, IntoCell, List, render_root};

pub fn render_profile_select<B>(frame: &mut Frame<B>, state: &SystemState)
where
    B: Backend,
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
