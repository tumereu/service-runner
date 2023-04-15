use tui::Frame;
use tui::backend::Backend;


use crate::client_state::ClientState;
use crate::ui::UIState;
use crate::ui::widgets::{Cell, List, render_root, Align, IntoCell};

pub fn render_profile_select<B>(
    frame: &mut Frame<B>,
    state: &ClientState,
) where B : Backend {
    let selected_idx = match &state.ui {
        UIState::ProfileSelect { selected_idx } => selected_idx,
        any @ _ => panic!("Invalid UI state in render_profile_select: {any:?}")
    };

    render_root(
        Cell {
            align_vert: Align::Center,
            align_horiz: Align::Center,
            element: List {
                items: List::simple_items(
                    state.config.profiles.iter().map(|prof| prof.name.clone()).collect()
                ),
                selection: *selected_idx
            }.into_el(),
            ..Default::default()
        },
        frame
    );
}
