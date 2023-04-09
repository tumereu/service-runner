use tui::backend::Backend;
use tui::Frame;
use tui::style::Color;

use crate::client_state::ClientState;
use crate::ui::UIState;
use crate::ui::widgets::{Flex, FlexAlign, FlexElement, FlexSize, List, render_root, Text, IntoFlexElement};

pub fn render_view_profile<B>(
    frame: &mut Frame<B>,
    state: &ClientState,
) where B : Backend {
    match &state.ui {
        UIState::ViewProfile { .. } => {},
        any @ _ => panic!("Invalid UI state in render_view_profile: {any:?}")
    };

    let profile = state.system_state.as_ref().map(|it| it.current_profile.as_ref()).flatten();

    if let Some(profile) = profile {
        render_root(
            Flex::new(vec![
                Flex::new(vec![
                    Text::from(&profile.name)
                        .into_flex()
                ]).bg(Color::Green)
                    .into_flex()
                    .align_vert(FlexAlign::Start)
                    .size_horiz(FlexSize::Grow)
                    .size_vert(FlexSize::Fixed(1)),
            ]),
            frame
        );
    }
}
