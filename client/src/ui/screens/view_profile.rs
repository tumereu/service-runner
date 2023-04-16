use std::cmp::{max, min};
use std::collections::HashMap;
use tui::backend::Backend;
use tui::Frame;
use tui::style::Color;
use shared::message::models::{CompileStatus, Profile, ServiceStatus};

use crate::client_state::ClientState;
use crate::ui::state::ViewProfilePane;
use crate::ui::UIState;
use crate::ui::widgets::{Flow, Cell, Align, List, render_root, Text, Dir, Spinner, IntoCell};
use crate::ui::widgets::Dir::{LeftRight, UpDown};

pub fn render_view_profile<B>(
    frame: &mut Frame<B>,
    state: &ClientState,
) where B: Backend {
    let (pane, selection) = match &state.ui {
        UIState::ViewProfile { active_pane, service_selection } => (active_pane, service_selection),
        any @ _ => panic!("Invalid UI state in render_view_profile: {any:?}")
    };

    let profile = state.system_state.as_ref().map(|it| it.current_profile.as_ref()).flatten();
    let service_statuses = state.system_state.as_ref().map(|it| &it.service_statuses);
    let active_border_color = Color::Rgb(180, 180, 0);
    let border_color = Color::Rgb(100, 100, 0);

    let service_selection: Option<usize> = match pane {
        ViewProfilePane::ServiceList => Some(*selection),
        _ => None
    };

    if let (Some(profile), Some(service_statuses)) = (profile, service_statuses) {
        let side_panel_width = min(40, max(25, frame.size().width / 5));

        render_root(Flow {
            direction: LeftRight,
            cells: vec![
                // List of services in the current profile
                Cell {
                    border: (
                        match pane {
                            ViewProfilePane::ServiceList => active_border_color.clone(),
                            _ => border_color.clone()
                        },
                        profile.name.clone()
                    ).into(),
                    min_width: side_panel_width,
                    element: service_list(profile, service_selection, service_statuses).into_el(),
                    ..Default::default()
                },
                // Output pane
                Cell {
                    border: (
                        match pane {
                            ViewProfilePane::OutputPane => active_border_color.clone(),
                            _ => border_color.clone()
                        },
                        String::from("Output")
                    ).into(),
                    fill: true,
                    align_vert: Align::Stretch,
                    align_horiz: Align::Stretch,
                    element: Flow {
                        direction: UpDown,
                        cells: state.output_store.query_lines(
                            frame.size().height.saturating_sub(1).into(),
                            None
                        ).into_iter()
                            .map(|(key, line)| {
                                Cell {
                                    align_horiz: Align::Start,
                                    element: Text {
                                        text: line.to_string(),
                                        ..Default::default()
                                    }.into_el(),
                                    ..Default::default()
                                }
                            }).collect(),
                        ..Default::default()
                    }.into_el(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }, frame);
    }
}

fn service_list(profile: &Profile, selection: Option<usize>, service_statuses: &HashMap<String, ServiceStatus>) -> List {
    let service_selection = 0;

    List {
        selection: selection.unwrap_or(usize::MAX),
        items: profile.services.iter()
            .enumerate()
            .map(|(_index, service)| {
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
                    align_horiz: Align::Stretch,
                    element: Flow {
                        direction: Dir::LeftRight,
                        cells: vec![
                            // Service name
                            Cell {
                                fill: true,
                                element: Text {
                                    text: service.name().to_string(),
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
