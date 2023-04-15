use std::cmp::{max, min};
use tui::backend::Backend;
use tui::Frame;
use tui::style::Color;
use shared::message::models::CompileStatus;

use crate::client_state::ClientState;
use crate::ui::UIState;
use crate::ui::widgets::{CellLayout, Cell, Align, List, render_root, Text, IntoFlexElement, Dir, Container, Spinner, IntoCell};
use crate::ui::widgets::Align::Stretch;
use crate::ui::widgets::Dir::UpDown;

pub fn render_view_profile<B>(
    frame: &mut Frame<B>,
    state: &ClientState,
) where B: Backend {
    match &state.ui {
        UIState::ViewProfile { .. } => {}
        any @ _ => panic!("Invalid UI state in render_view_profile: {any:?}")
    };

    let profile = state.system_state.as_ref().map(|it| it.current_profile.as_ref()).flatten();
    let service_statuses = state.system_state.as_ref().map(|it| &it.service_statuses);

    if let (Some(profile), Some(service_statuses)) = (profile, service_statuses) {
        let side_panel_width = min(40, max(25, frame.size().width / 5));

        let service_selection = 0;

        render_root(CellLayout {
            direction: UpDown,
            cells: vec![
                Cell {
                    element: Cell {
                        padding_left: 2,
                        element: Text {
                            text: profile.name.to_owned(),
                            ..Default::default()
                        }.into_el(),
                        ..Default::default()
                    }.into_el(),
                    align_vert: Align::Stretch,
                    bg: Some(Color::Green),
                    ..Default::default()
                },
                // Split the panel for side panel/service selection and output window
                Cell {
                    fill: true,
                    align_vert: Stretch,
                    align_horiz: Stretch,
                    element: CellLayout {
                        direction: Dir::LeftRight,
                        cells: vec![
                            // List of services in the current profile
                            Cell {
                                padding_left: 1,
                                padding_top: 1,
                                padding_bottom: 1,
                                padding_right: 1,
                                min_width: side_panel_width,
                                fill: true,
                                element: service_list().into_el(),
                                ..Default::default()
                            },
                            // Output window. TODO
                            Cell {
                                fill: true,
                                ..Default::default()
                            },
                        ],
                        ..Default::default()
                    }.into_el(),
                    ..Default::default()
                }
            ],
            ..Default::default()
        }, frame);
    }
}

fn service_list() -> CellLayout {
    CellLayout {
        cells: profile.services.iter()
            .enumerate()
            .map(|(index, service)| {
                let status = service_statuses.get(service.name());
                let show_output = status.map(|it| it.show_output).unwrap_or(false);
                let should_run = status.map(|it| it.should_run).unwrap_or(false);
                let auto_recompile = status.map(|it| it.auto_recompile).unwrap_or(false);
                let is_running = status.map(|it| it.is_running).unwrap_or(false);
                let is_compiling = status.map(|it| {
                    match it.compile_status {
                        CompileStatus::Compiling(_) => true,
                        _ => false
                    }
                }).unwrap_or(false);

                Cell {
                    fill: true,
                    bg: if service_selection == index {
                        Some(Color::Blue)
                    } else {
                        None
                    },
                    element: CellLayout {
                        direction: Dir::LeftRight,
                        cells: vec![
                            // Service name
                            Cell {
                                fill: true,
                                element: Text {
                                    text: service.name(),
                                    ..Default::default()
                                }.into_el(),
                                ..Default::default()
                            },
                            // Status prefix
                            Cell {
                                element: Text {
                                    text: " [".into(),
                                    ..Default::default()
                                }.into_el(),
                                ..Default::default()
                            },
                            // Run status
                            Cell {
                                element: Text {
                                    text: "R".into(),
                                    fg: if !should_run {
                                        Color::Gray
                                    } else if is_running {
                                        Color::Green
                                    } else {
                                        Color::Yellow
                                    }.into(),
                                    ..Default::default()
                                }.into_el(),
                                ..Default::default()
                            },
                            // Compilation status
                            Cell {
                                element: Text {
                                    text: "C".into(),
                                    fg: if !auto_recompile {
                                        Color::Gray
                                    } else if is_compiling {
                                        Color::Yellow
                                    } else {
                                        Color::Green
                                    }.into(),
                                    ..Default::default()
                                }.into_el(),
                                ..Default::default()
                            },
                            // Output status
                            Cell {
                                element: Text {
                                    text: "C".into(),
                                    fg: if show_output {
                                        Color::Green
                                    } else {
                                        Color::Gray
                                    }.into(),
                                    ..Default::default()
                                }.into_el(),
                                ..Default::default()
                            },
                            // Status suffix
                            Cell {
                                element: Text {
                                    text: "]".into(),
                                    ..Default::default()
                                }.into_el(),
                                ..Default::default()
                            },
                            Cell {
                                padding_left: 1,
                                element: Spinner {
                                    active: is_compiling,
                                    ..Default::default()
                                }.into_el(),
                                ..Default::default()
                            },
                        ],
                        ..Default::default()
                    }.into_el(),
                    ..Default::default()
                }
            }).collect(),
        ..Default::default()
    }
}
