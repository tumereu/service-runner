use std::cell::RefCell;
use std::cmp::{max, min};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::format;
use std::hash::{Hash, Hasher};
use std::iter;
use std::rc::Rc;
use once_cell::sync::Lazy;

use tui::backend::Backend;
use tui::style::Color;
use tui::Frame;
use tui::layout::Rect;

use crate::models::action::models::{AutoCompileMode, CompileStatus, get_active_outputs, OutputKey, OutputKind, Profile, RunStatus, ServiceAction, ServiceStatus};

use crate::system_state::SystemState;
use crate::ui::state::{ViewProfilePane, ViewProfileState};
use crate::ui::widgets::{render_root, Align, Cell, Dir, Flow, IntoCell, List, Spinner, Text, OutputDisplay, OutputLine, LinePart, Renderable, render_at_pos, Toggle};
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

pub fn render_view_profile<B>(frame: &mut Frame<B>, state: &SystemState)
where
    B: Backend,
{
    let (pane, selection, wrap_output, output_pos_horiz, output_pos_vert, floating_pane) = match &state.ui {
        &CurrentScreen::ViewProfile(ViewProfileState {
            active_pane,
            service_selection,
            wrap_output,
            output_pos_horiz,
            output_pos_vert,
            floating_pane,
        }) => (active_pane, service_selection, wrap_output, output_pos_horiz, output_pos_vert, floating_pane),
        any @ _ => panic!("Invalid UI state in render_view_profile: {any:?}"),
    };

    let profile = state
        .runner_state
        .as_ref()
        .map(|it| it.current_profile.as_ref())
        .flatten();
    let service_statuses = state.runner_state.as_ref().map(|it| &it.service_statuses);

    // TODO move into a theme?
    let active_border_color = Color::Rgb(180, 180, 0);
    let border_color = Color::Rgb(100, 100, 0);

    let service_selection: Option<usize> = match pane {
        ViewProfilePane::ServiceList => Some(selection),
        _ => None,
    };

    if let (Some(profile), Some(service_statuses)) = (profile, service_statuses) {
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
                                ViewProfilePane::ServiceList => active_border_color.clone(),
                                _ => border_color.clone(),
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
                                ViewProfilePane::OutputPane => active_border_color.clone(),
                                _ => border_color.clone(),
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
                            &profile,
                            &state
                        ).into_el(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            frame,
        );

        match floating_pane {
            Some(ViewProfileFloatingPane::ServiceAutocompleteDetails { detail_list_selection }) => {
                let selected_service_bounds = selected_service_bounds.borrow();
                render_at_pos(
                    Cell {
                        border: (active_border_color, String::from("Autocomplete Details")).into(),
                        element: Flow {
                            cells: vec![
                                Cell {
                                    element: Toggle {
                                        options: vec![
                                            String::from("Automatic"),
                                            String::from("Disabled"),
                                            String::from("Custom"),
                                        ],
                                        selection: 0
                                    }.into_el(),
                                    ..Default::default()
                                }
                            ],
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
                                            (_, _) if service.run.is_none() => inactive_color.clone(),
                                            (RunStatus::Healthy | RunStatus::Running, _) if !status.should_run => {
                                                processing_color.clone()
                                            },
                                            (_, _) if !status.should_run => inactive_color.clone(),
                                            (RunStatus::Failed, _) => error_color.clone(),
                                            (_, ServiceAction::Restart) => processing_color.clone(),
                                            (RunStatus::Healthy, _) => active_color.clone(),
                                            (RunStatus::Running, _) => processing_color.clone(),
                                            (RunStatus::Stopped, ServiceAction::Recompile) => processing_color.clone(),
                                            (_, _) if status.should_run => processing_color.clone(),
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
                                            ) => processing_color.clone(),
                                            CompileStatus::None => inactive_color.clone(),
                                            CompileStatus::FullyCompiled => active_color.clone(),
                                            CompileStatus::PartiallyCompiled(_) => processing_color.clone(),
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
                                    fg: if let Some(status) = status {
                                        match &status.auto_compile {
                                            None => inactive_color.clone(),
                                            Some(AutoCompileMode::AUTOMATIC) => active_color.clone(),
                                            Some(AutoCompileMode::CUSTOM) if status.has_uncompiled_changes => {
                                                processing_color.clone()
                                            },
                                            Some(AutoCompileMode::CUSTOM) => secondary_active_color.clone(),
                                            Some(AutoCompileMode::DISABLED) => inactive_color.clone(),
                                        }
                                    } else {
                                        inactive_color.clone()
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
                            get_active_outputs(&state.output_store, &state.runner_state)
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
                                        }.into(),
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
            }.into_iter()
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