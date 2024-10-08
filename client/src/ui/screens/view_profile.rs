use std::cell::RefCell;
use std::cmp::{max, min};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::iter;
use std::rc::Rc;
use itertools::Itertools;

use once_cell::sync::Lazy;

use tui::backend::Backend;
use tui::style::Color;
use tui::Frame;
use tui::layout::Rect;
use crate::models::{AutomationMode, CompileStatus, get_active_outputs, OutputKey, OutputKind, Profile, RunStatus, ServiceAction, ServiceStatus};
use crate::models::AutomationMode::{Automatic, Disabled};
use crate::system_state::SystemState;
use crate::ui::state::{ViewProfilePane, ViewProfileState};
use crate::ui::widgets::{render_root, Align, Cell, Dir, Flow, IntoCell, List, Spinner, Text, OutputDisplay, OutputLine, LinePart, render_at_pos};
use crate::ui::{CurrentScreen, ViewProfileFloatingPane};

const SERVICE_NAME_COLORS: Lazy<Vec<Color>> = Lazy::new(|| {
    vec![
        Color::Rgb(255, 0, 0),
        Color::Rgb(255, 165, 0),
        Color::Rgb(255, 255, 0),
        Color::Rgb(0, 255, 0),
        Color::Rgb(0, 255, 255),
        Color::Rgb(0, 120, 180),
        Color::Rgb(128, 0, 128),
        Color::Rgb(255, 0, 255),
        Color::Rgb(255, 192, 203),
        Color::Rgb(255, 215, 0),
        Color::Rgb(255, 69, 0),
        Color::Rgb(0, 128, 0),
        Color::Rgb(139, 0, 139),
    ]
});

pub fn render_view_profile<B>(frame: &mut Frame<B>, system: &SystemState)
where
    B: Backend,
{
    let (pane, selection, wrap_output, output_pos_horiz, output_pos_vert, floating_pane) = match &system.ui.screen {
        &CurrentScreen::ViewProfile(ViewProfileState {
            active_pane,
            service_selection,
            wrap_output,
            output_pos_horiz,
            output_pos_vert,
            floating_pane,
        }) => (active_pane, service_selection, wrap_output, output_pos_horiz, output_pos_vert, floating_pane),
        any => panic!("Invalid UI state in render_view_profile: {any:?}"),
    };

    let profile = &system.current_profile;
    let service_statuses = &system.service_statuses;

    // TODO move into a theme?
    let active_border_color = Color::Rgb(180, 180, 0);
    let border_color = Color::Rgb(100, 100, 0);
    let active_color = Color::Rgb(0, 140, 0);
    let secondary_active_color = Color::Rgb(0, 40, 180);

    let service_selection: Option<usize> = match pane {
        ViewProfilePane::ServiceList => Some(selection),
        _ => None,
    };

    if let Some(profile) = profile {
        let side_panel_width = min(40, max(25, frame.size().width / 5));
        let (service_list, selected_service_bounds) = service_list(profile, service_selection, service_statuses);

        render_root(
            Flow {
                direction: Dir::LeftRight,
                cells: vec![
                    // List of services in the current profile
                    Cell {
                        border: (
                            match pane {
                                ViewProfilePane::ServiceList => active_border_color,
                                _ => border_color,
                            },
                            profile.name.clone(),
                        )
                            .into(),
                        min_width: side_panel_width,
                        align_horiz: Align::Stretch,
                        element: service_list.into_el(),
                        ..Default::default()
                    },
                    // Output pane
                    Cell {
                        border: (
                            match pane {
                                ViewProfilePane::OutputPane => active_border_color,
                                _ => border_color,
                            },
                            format!(
                                "Output [wrap: {wrap_symbol}]",
                                wrap_symbol = if wrap_output {
                                    "Y"
                                } else {
                                    "N"
                                }
                            ),
                        )
                            .into(),
                        fill: true,
                        align_vert: Align::Stretch,
                        align_horiz: Align::Stretch,
                        element: output_pane(
                            frame.size().height.into(),
                            wrap_output,
                            output_pos_horiz,
                            output_pos_vert,
                            profile,
                            system
                        ).into_el(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            frame,
        );

        match floating_pane {
            Some(ViewProfileFloatingPane::ServiceAutomationDetails { detail_list_selection: _ }) => {
                let selected_service_bounds = selected_service_bounds.borrow();
                let automation_modes: Vec<(String, AutomationMode)> = system.iter_services_with_statuses()
                    .dropping(service_selection.unwrap_or(0))
                    .next()
                    .map(|(service, status)| {
                        service.automation.iter()
                            .map(|automation_entry| {
                                let automation_name = automation_entry.name.clone();
                                let current_mode = status.automation_modes.get(&automation_name).copied()
                                    .unwrap_or(AutomationMode::Disabled);

                                (automation_name, current_mode)
                            }).collect()
                    }).unwrap_or_default();
                let longest_name: u16 = automation_modes.iter().map(|(name, _)| name.len() as u16).max().unwrap_or(0);

                render_at_pos(
                    Cell {
                        border: (active_border_color, String::from("Automation")).into(),
                        opaque: true,
                        element: List {
                            selection: 0,
                            items: automation_modes.iter().map(|(name, mode)| {
                                Cell {
                                    element: Flow {
                                        direction: Dir::LeftRight,
                                        cells: vec![
                                            Cell {
                                                element: Text {
                                                    text: name.clone(),
                                                    ..Default::default()
                                                }.into_el(),
                                                min_width: longest_name + 3,
                                                padding_right: 3,
                                                ..Default::default()
                                            },
                                            Cell {
                                                element: Text {
                                                    text: match mode {
                                                        AutomationMode::Disabled => String::from("Disabled"),
                                                        AutomationMode::Triggerable => String::from("Triggerable"),
                                                        AutomationMode::Automatic => String::from("Automatic"),
                                                    },
                                                    fg: match mode {
                                                        AutomationMode::Disabled => None,
                                                        AutomationMode::Triggerable => Some(secondary_active_color),
                                                        AutomationMode::Automatic => Some(active_color),
                                                    },
                                                    ..Default::default()
                                                }.into_el(),
                                                align_horiz: Align::End,
                                                min_width: 11,
                                                ..Default::default()
                                            },
                                        ],
                                        ..Default::default()
                                    }.into_el(),
                                    ..Default::default()
                                }
                            }).collect(),
                            ..Default::default()
                        }.into_el(),
                        ..Default::default()
                    },
                    // Render the floating pane after the selected service but on the same level
                    (
                        selected_service_bounds.x + selected_service_bounds.width,
                        selected_service_bounds.y,
                    ),
                    frame,
                );
            },
            _ => {}
        }
    }
}

fn service_list(
    profile: &Profile,
    selection: Option<usize>,
    service_statuses: &HashMap<String, ServiceStatus>,
) -> (List, Rc<RefCell<Rect>>) {
    // TODO Theme?
    let active_color = Color::Rgb(0, 140, 0);
    let secondary_active_color = Color::Rgb(0, 40, 180);
    let processing_color = Color::Rgb(230, 180, 0);
    let error_color = Color::Rgb(180, 0, 0);
    let inactive_color = Color::Gray;
    let selected_service_bounds = Rc::new(RefCell::new(Rect::new(0, 0, 0, 0)));

    let list = List {
        selection: selection.unwrap_or(usize::MAX),
        items: profile
            .services
            .iter()
            .enumerate()
            .map(|(index, service)| {
                let status = service_statuses.get(&service.name);
                let show_output = status.map(|it| it.show_output).unwrap_or(false);
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
                    store_bounds: if index == selection.unwrap_or(usize::MAX) {
                        Some(selected_service_bounds.clone())
                    } else {
                        None
                    },
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
                                    text: if service.run.is_none() {
                                        "-"
                                    } else if status.map(|status| status.debug).unwrap_or(false) {
                                        "D"
                                    } else {
                                        "R"
                                    }.into(),
                                    fg: if let Some(status) = status {
                                        match (&status.run_status, &status.action) {
                                            (_, _) if service.run.is_none() => inactive_color,
                                            (RunStatus::Healthy | RunStatus::Running, _) if !status.should_run => {
                                                processing_color
                                            },
                                            (_, _) if !status.should_run => inactive_color,
                                            (RunStatus::Failed, _) => error_color,
                                            (_, ServiceAction::Restart) => processing_color,
                                            (RunStatus::Healthy, _) => active_color,
                                            (RunStatus::Running, _) => processing_color,
                                            (RunStatus::Stopped, ServiceAction::Recompile) => processing_color,
                                            (_, _) if status.should_run => processing_color,
                                            (_, _) => inactive_color,
                                        }
                                    } else {
                                        inactive_color
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
                                    text: if service.compile.is_some() {
                                        "C"
                                    } else {
                                        "-"
                                    }.into(),
                                    fg: if let Some(status) = status {
                                        match status.compile_status {
                                            _ if matches!(
                                                status.action,
                                                ServiceAction::Recompile
                                            ) => processing_color,
                                            CompileStatus::None => inactive_color,
                                            CompileStatus::FullyCompiled => active_color,
                                            CompileStatus::PartiallyCompiled(_) => processing_color,
                                            CompileStatus::Compiling(_) => processing_color,
                                            CompileStatus::Failed => error_color,
                                        }
                                    } else {
                                        inactive_color
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
                                        active_color
                                    } else {
                                        inactive_color
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
                                    text: if !service.automation.is_empty() {
                                        "A"
                                    } else {
                                        "-"
                                    }.into(),

                                    fg: if let Some(status) = status {
                                        if service.automation.is_empty() {
                                            inactive_color
                                        } else if !status.automation_enabled {
                                            inactive_color
                                        } else if !status.pending_automations.is_empty() {
                                            processing_color
                                        } else if status.automation_modes.iter().all(|(_, mode)| *mode == Automatic) {
                                            active_color
                                        } else if status.automation_modes.iter().all(|(_, mode)| *mode == Disabled) {
                                            inactive_color
                                        } else {
                                            secondary_active_color
                                        }
                                    } else {
                                        inactive_color
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
    };

    (list, selected_service_bounds)
}

fn output_pane(
    height: usize,
    wrap_output: bool,
    pos_horiz: Option<u64>,
    pos_vert: Option<u128>,
    profile: &Profile,
    state: &SystemState,
) -> Flow {
    Flow {
        direction: Dir::UpDown,
        cells: iter::once(
            Cell {
                align_horiz: Align::Stretch,
                align_vert: Align::Stretch,
                fill: true,
                element: OutputDisplay {
                    wrap: wrap_output,
                    pos_horiz,
                    lines: state.output_store
                        .query_lines_to(
                            height,
                            pos_vert,
                            &get_active_outputs(&state.output_store, state)
                        )
                        .into_iter()
                        .map(|(key, line)| {
                            let service_idx = profile.services.iter()
                                .enumerate()
                                .find(|(_, service)| service.name == key.service_ref)
                                .unwrap().0;

                            OutputLine {
                                prefix: vec![
                                    LinePart {
                                        text: match key.kind {
                                            OutputKind::Run => "r/",
                                            OutputKind::Compile => "c/"
                                        }.to_string(),
                                        color: match key.kind {
                                            OutputKind::Run => Color::Rgb(0, 180, 0),
                                            OutputKind::Compile => Color::Rgb(0, 120, 220)
                                        }.into(),
                                    },
                                    LinePart {
                                        text: format!("{name}/", name = key.name),
                                        color: match key.name.as_str() {
                                            OutputKey::STD => None,
                                            OutputKey::CTL => Some(Color::Rgb(180, 0, 130)),
                                            other => Some(hashed_color(other))
                                        },
                                    },
                                    LinePart {
                                        text: format!("{service} | ", service = key.service_ref.clone()),
                                        color: SERVICE_NAME_COLORS[service_idx % SERVICE_NAME_COLORS.len()].into(),
                                    },
                                ],
                                parts: vec![
                                    LinePart {
                                        text: line.value.clone(),
                                        color: None
                                    }
                                ]
                            }
                        }).collect(),
                }.into_el(),
                ..Default::default()
            }
        ).chain(
            if pos_vert.is_none() {
                Some(
                    Cell {
                        align_horiz: Align::Stretch,
                        element: Spinner {
                            active: true,
                            ..Default::default()
                        }.into_el(),
                        ..Default::default()
                    }
                )
            } else {
                None
            }
        )
            .collect(),
        ..Default::default()
    }
}

fn hashed_color(text: &str) -> Color {
    // Hash the service name to obtain a color for it
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    let hash: usize = hasher.finish() as usize;
    SERVICE_NAME_COLORS[hash % SERVICE_NAME_COLORS.len()]
}