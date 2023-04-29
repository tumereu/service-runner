use std::cmp::{max, min};
use std::collections::HashMap;

use tui::backend::Backend;
use tui::style::Color;
use tui::Frame;

use shared::message::models::{AutoCompileMode, CompileStatus, Profile, RunStatus, ServiceAction, ServiceStatus};

use crate::client_state::ClientState;
use crate::ui::state::ViewProfilePane;
use crate::ui::widgets::{render_root, Align, Cell, Dir, Flow, IntoCell, List, Spinner, Text};
use crate::ui::UIState;

pub fn render_view_profile<B>(frame: &mut Frame<B>, state: &ClientState)
where
    B: Backend,
{
    let (pane, selection) = match &state.ui {
        UIState::ViewProfile {
            active_pane,
            service_selection,
        } => (active_pane, service_selection),
        any @ _ => panic!("Invalid UI state in render_view_profile: {any:?}"),
    };

    let profile = state
        .system_state
        .as_ref()
        .map(|it| it.current_profile.as_ref())
        .flatten();
    let service_statuses = state.system_state.as_ref().map(|it| &it.service_statuses);

    // TODO move into a theme?
    let active_border_color = Color::Rgb(180, 180, 0);
    let border_color = Color::Rgb(100, 100, 0);

    let service_selection: Option<usize> = match pane {
        ViewProfilePane::ServiceList => Some(*selection),
        _ => None,
    };

    if let (Some(profile), Some(service_statuses)) = (profile, service_statuses) {
        let side_panel_width = min(40, max(25, frame.size().width / 5));

        render_root(
            Flow {
                direction: Dir::LeftRight,
                cells: vec![
                    // List of services in the current profile
                    Cell {
                        border: (
                            match pane {
                                ViewProfilePane::ServiceList => active_border_color.clone(),
                                _ => border_color.clone(),
                            },
                            profile.name.clone(),
                        )
                            .into(),
                        min_width: side_panel_width,
                        element: service_list(profile, service_selection, service_statuses)
                            .into_el(),
                        ..Default::default()
                    },
                    // Output pane
                    Cell {
                        border: (
                            match pane {
                                ViewProfilePane::OutputPane => active_border_color.clone(),
                                _ => border_color.clone(),
                            },
                            String::from("Output"),
                        )
                            .into(),
                        fill: true,
                        align_vert: Align::Stretch,
                        align_horiz: Align::Stretch,
                        element: Flow {
                            direction: Dir::UpDown,
                            cells: state
                                .output_store
                                .query_lines(frame.size().height.saturating_sub(2).into(), None)
                                .into_iter()
                                .map(|(_key, line)| Cell {
                                    align_horiz: Align::Start,
                                    element: Text {
                                        text: line.to_string(),
                                        ..Default::default()
                                    }
                                    .into_el(),
                                    ..Default::default()
                                })
                                .collect(),
                            ..Default::default()
                        }
                        .into_el(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            frame,
        );
    }
}

fn service_list(
    profile: &Profile,
    selection: Option<usize>,
    service_statuses: &HashMap<String, ServiceStatus>,
) -> List {
    // TODO Theme?
    let active_color = Color::Rgb(0, 140, 0);
    let secondary_active_color = Color::Rgb(0, 0, 140);
    let processing_color = Color::Rgb(230, 180, 0);
    let error_color = Color::Rgb(180, 0, 0);
    let inactive_color = Color::Gray;

    List {
        selection: selection.unwrap_or(usize::MAX),
        items: profile
            .services
            .iter()
            .enumerate()
            .map(|(_index, service)| {
                let status = service_statuses.get(&service.name);
                let show_output = status.map(|it| it.show_output).unwrap_or(false);
                let auto_recompile = status.map(|it| it.auto_recompile).unwrap_or(false);
                let is_processing = status
                    .map(|it| match (&it.compile_status, &it.run_status) {
                        (CompileStatus::Compiling(_), _) => true,
                        (CompileStatus::PartiallyCompiled(_), _) => true,
                        (_, RunStatus::Running) => true,
                        _ => false,
                    })
                    .unwrap_or(false);

                Cell {
                    align_horiz: Align::Stretch,
                    element: Flow {
                        direction: Dir::LeftRight,
                        cells: vec![
                            // Service name
                            Cell {
                                fill: true,
                                element: Text {
                                    text: service.name.to_string(),
                                    ..Default::default()
                                }
                                .into_el(),
                                ..Default::default()
                            },
                            // Status prefix
                            Cell {
                                element: Text {
                                    text: " ".into(),
                                    ..Default::default()
                                }
                                .into_el(),
                                ..Default::default()
                            },
                            // Run status
                            Cell {
                                element: Text {
                                    text: if service.run.is_none() { "-" } else { "R" }.into(),
                                    fg: if let Some(status) = status {
                                        match (&status.run_status, &status.action) {
                                            (_, _) if service.run.is_none() => inactive_color.clone(),
                                            (_, ServiceAction::Restart) => processing_color.clone(),
                                            (RunStatus::Healthy, _) => active_color.clone(),
                                            (RunStatus::Running, _) => processing_color.clone(),
                                            (RunStatus::Failed, _) => error_color.clone(),
                                            (RunStatus::Stopped, ServiceAction::Recompile) => processing_color.clone(),
                                            (RunStatus::Stopped, _) if status.should_run => processing_color.clone(),
                                            (_, _) => inactive_color.clone(),
                                        }
                                    } else {
                                        inactive_color.clone()
                                    }
                                    .into(),
                                    ..Default::default()
                                }
                                .into_el(),
                                ..Default::default()
                            },
                            // Compilation status
                            Cell {
                                element: Text {
                                    text: "C".into(),
                                    fg: if let Some(status) = status {
                                        match status.compile_status {
                                            _ if matches!(
                                                status.action,
                                                ServiceAction::Recompile
                                            ) =>
                                            {
                                                processing_color.clone()
                                            }
                                            CompileStatus::None => {
                                                if auto_recompile {
                                                    active_color.clone()
                                                } else {
                                                    inactive_color.clone()
                                                }
                                            }
                                            CompileStatus::FullyCompiled => {
                                                if auto_recompile {
                                                    active_color.clone()
                                                } else {
                                                    inactive_color.clone()
                                                }
                                            }
                                            CompileStatus::PartiallyCompiled(_) => {
                                                processing_color.clone()
                                            }
                                            CompileStatus::Compiling(_) => processing_color.clone(),
                                            CompileStatus::Failed => error_color.clone(),
                                        }
                                    } else {
                                        inactive_color.clone()
                                    }
                                    .into(),
                                    ..Default::default()
                                }
                                .into_el(),
                                ..Default::default()
                            },
                            // Output status
                            Cell {
                                element: Text {
                                    text: "O".into(),
                                    fg: if show_output {
                                        active_color.clone()
                                    } else {
                                        inactive_color.clone()
                                    }
                                    .into(),
                                    ..Default::default()
                                }
                                .into_el(),
                                ..Default::default()
                            },
                            // Autocompile status
                            Cell {
                                element: Text {
                                    text: if service.autocompile.is_some() {
                                        "A"
                                    } else {
                                        "-"
                                    }.into(),
                                    fg: match service.autocompile.as_ref().map(|autocompile| &autocompile.mode) {
                                        None => inactive_color.clone(),
                                        Some(AutoCompileMode::AUTOMATIC) => active_color.clone(),
                                        Some(AutoCompileMode::TRIGGERED) => secondary_active_color.clone(),
                                        Some(AutoCompileMode::DISABLED) => inactive_color.clone(),
                                    }.into(),
                                    ..Default::default()
                                }
                                    .into_el(),
                                ..Default::default()
                            },
                            Cell {
                                padding_left: 1,
                                element: Spinner {
                                    active: is_processing,
                                    ..Default::default()
                                }
                                .into_el(),
                                ..Default::default()
                            },
                        ],
                        ..Default::default()
                    }
                    .into_el(),
                    ..Default::default()
                }
            })
            .collect(),
        ..Default::default()
    }
}
