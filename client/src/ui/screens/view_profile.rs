use std::cmp::{max, min};
use tui::backend::Backend;
use tui::Frame;
use tui::style::Color;

use crate::client_state::ClientState;
use crate::ui::UIState;
use crate::ui::widgets::{Flex, FlexElement, FlexSize, List, render_root, Text, IntoFlexElement, FlexDir, Container};

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
        let side_panel_width = min(40, max(25, frame.size().width / 5));

        let service_selection = 0;

        render_root(
            Flex::new(vec![
                // Display profile name in the title
                Container::from(
                    Flex::new(vec![
                        Text::from(&profile.name)
                            .into_flex()
                    ])
                ).bg(Color::Green)
                    .into_flex()
                    .size_horiz(FlexSize::Grow)
                    .size_vert(FlexSize::Wrap),
                // Split the panel for side panel/service selection and output window
                Flex::new(vec![
                    // List of services in the current profile
                    Container::from(
                        Flex::new(
                            profile.services.iter()
                                .enumerate()
                                .map(|(index, service)| {
                                    Flex::new(vec![
                                        Container::from(
                                            Text::from(service.name())
                                        ).into_flex().grow_horiz(),
                                        Text::from("[R]").into_flex()
                                    ]).direction(FlexDir::LeftRight)
                                        .bg(
                                            if service_selection == index {
                                                Some(Color::Blue)
                                            } else {
                                                None
                                            }
                                        ).into_flex().grow_horiz()
                                }).collect()
                        )
                    ).pad_left(1).pad_top(1).min_width(side_panel_width)
                        .into_flex()
                        .grow_horiz()
                ]).direction(FlexDir::LeftRight).into_flex(),
                // Output window. TODO
                List::new().into_flex().grow_horiz().grow_vert()
            ]),
            frame
        );
    }
}
