use tui::backend::Backend;
use tui::Frame;
use tui::style::Color;

use crate::client_state::ClientState;
use crate::ui::UIState;
use crate::ui::widgets::{Flex, FlexElement, FlexSize, List, render_root, Text, IntoFlexElement, Styleable, FlexDir};

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
        let longest_service_name = profile.services.iter()
            .map(|service| service.name().len())
            .max()
            .unwrap_or(0);

        render_root(
            Flex::new(vec![
                // Display profile name in the title
                Flex::new(vec![
                    Text::from(&profile.name)
                        .into_flex()
                ]).styling().bg(Color::Green)
                    .into_flex()
                    .size_horiz(FlexSize::Grow)
                    .size_vert(FlexSize::Wrap),
                // List of services in the current profile
                List::new()
                    .items(
                        profile.services.iter()
                            .map(|service| {
                                Flex::new(vec![
                                    Text::from(service.name())
                                        .ljust(longest_service_name + 3)
                                        .into_flex(),
                                    Text::from("[R]").into_flex()
                                ]).direction(FlexDir::LeftRight)
                                    .into()
                            }).collect()
                    )
                    .styling().pad_left(1).pad_top(1)
                    .into_flex()
                    .grow_horiz()
            ]),
            frame
        );
    }
}
