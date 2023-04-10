use std::cmp::{max, min};
use tui::backend::Backend;
use tui::Frame;
use tui::style::Color;

use crate::client_state::ClientState;
use crate::ui::UIState;
use crate::ui::widgets::{Flex, FlexElement, FlexSize, List, render_root, Text, IntoFlexElement, FlexDir, Container, Spinner};

pub fn render_view_profile<B>(
    frame: &mut Frame<B>,
    state: &ClientState,
) where B : Backend {
    match &state.ui {
        UIState::ViewProfile { .. } => {},
        any @ _ => panic!("Invalid UI state in render_view_profile: {any:?}")
    };

    let profile = state.system_state.as_ref().map(|it| it.current_profile.as_ref()).flatten();
    let service_statuses = state.system_state.as_ref().map(|it| &it.service_statuses);

    if let (Some(profile), Some(service_statuses)) = (profile, service_statuses) {
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
                                    let status = service_statuses.get(service.name());
                                    let show_output = status.map(|it| it.show_output).unwrap_or(false);
                                    let should_run = status.map(|it| it.should_run).unwrap_or(false);
                                    let auto_recompile = status.map(|it| it.auto_recompile).unwrap_or(false);
                                    let is_running = status.map(|it| it.is_running).unwrap_or(false);
                                    let is_compiling = status.map(|it| it.is_compiling).unwrap_or(false);

                                    Flex::new(vec![
                                        Container::from(
                                            Text::from(service.name())
                                        ).into_flex().grow_horiz(),
                                        // Run status
                                        Text::from(" [").into_flex(),
                                        Text::from("R").fg(
                                            if !should_run {
                                                Color::Gray
                                            } else if is_running {
                                                Color::Green
                                            } else {
                                                Color::Yellow
                                            }.into()
                                        ).into_flex(),
                                        // Compilation status
                                        Text::from("C").fg(
                                            if !auto_recompile {
                                                Color::Gray
                                            } else if is_compiling {
                                                Color::Yellow
                                            } else {
                                                Color::Green
                                            }.into()
                                        ).into_flex(),
                                        // Output status
                                        Text::from("O").fg(
                                            if show_output {
                                                Color::Green
                                            } else {
                                                Color::Gray
                                            }.into()
                                        ).into_flex(),
                                        Text::from("]").into_flex(),
                                        // TODO: make active when work is actively performed for service
                                        Container::from(
                                            Spinner::new().active(false)
                                        ).pad_left(1).into_flex()
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
