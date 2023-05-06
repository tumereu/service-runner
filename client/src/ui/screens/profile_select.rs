use tui::backend::Backend;
use tui::Frame;
use tui::style::Color;

use crate::client_state::ClientState;
use crate::ui::widgets::{render_root, Align, Cell, IntoCell, List};
use crate::ui::UIState;

pub fn render_profile_select<B>(frame: &mut Frame<B>, state: &ClientState)
where
    B: Backend,
{
    let selected_idx = match &state.ui {
        UIState::ProfileSelect { selected_idx } => selected_idx,
        any @ _ => panic!("Invalid UI state in render_profile_select: {any:?}"),
    };

    // TODO theme?
    let active_border_color = Color::Rgb(180, 180, 0);

    render_root(
        Cell {
            align_vert: Align::Center,
            align_horiz: Align::Center,
            element: Cell {
                align_vert: Align::Center,
                align_horiz: Align::Center,
                border: Some((active_border_color, "Select profile".into())),
                min_width: 16,
                fill: false,
                element: List {
                    items: List::simple_items(
                        state
                            .config
                            .profiles
                            .iter()
                            .map(|prof| prof.name.clone())
                            .collect(),
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
