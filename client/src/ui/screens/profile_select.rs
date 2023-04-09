use tui::Frame;
use tui::backend::Backend;
use tui::style::Color;

use crate::client_state::ClientState;
use crate::ui::UIState;
use crate::ui::widgets::{Flex, FlexAlign, FlexElement, IntoFlexElement, List, render_root};

pub fn render_profile_select<B>(
    frame: &mut Frame<B>,
    state: &ClientState,
) where B : Backend {
    let selected_idx = match &state.ui {
        UIState::ProfileSelect { selected_idx } => selected_idx,
        any @ _ => panic!("Invalid UI state in render_profile_select: {any:?}")
    };

    render_root(
        Flex::new(vec![
           List::new().items(
                state.config.profiles.iter().map(|prof| prof.name.clone()).collect()
           ).selection(*selected_idx)
               .into_flex()
               .align_vert(FlexAlign::Center)
               .align_horiz(FlexAlign::Center),
        ]),
        frame
    );
}
