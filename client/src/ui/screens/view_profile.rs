use std::cell::RefCell;
use std::cmp::{max, min};
use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::iter;
use std::rc::Rc;

use itertools::Itertools;
use once_cell::sync::Lazy;
use ratatui::backend::Backend;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::Frame;

use crate::config::{AutomationMode, Block};
use crate::models::{get_active_outputs, BlockStatus, OutputKind, Profile, WorkStep};
use crate::system_state::SystemState;
use crate::ui::state::{ViewProfilePane, ViewProfileState};
use crate::ui::widgets::{
    render_at_pos, render_root, Align, Cell, Dir, Flow, IntoCell, LinePart, List, OutputDisplay,
    OutputLine, Spinner, Text,
};
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

pub fn render_view_profile(frame: &mut Frame, system: &SystemState)
{
    let (pane, selection, wrap_output, output_pos_horiz, output_pos_vert, floating_pane) =
        match &system.ui.screen {
            &CurrentScreen::ViewProfile(ViewProfileState {
                active_pane,
                service_selection,
                wrap_output,
                output_pos_horiz,
                output_pos_vert,
                floating_pane,
            }) => (
                active_pane,
                service_selection,
                wrap_output,
                output_pos_horiz,
                output_pos_vert,
                floating_pane,
            ),
            any => panic!("Invalid UI state in render_view_profile: {any:?}"),
        };

    let profile = &system.current_profile;

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
        let (service_list, selected_service_bounds) =
            service_list(system, profile, service_selection);

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
                            profile.definition.id.clone(),
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
                                wrap_symbol = if wrap_output { "Y" } else { "N" }
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
                            system,
                        )
                        .into_el(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            frame,
        );

        match floating_pane {
            Some(ViewProfileFloatingPane::ServiceAutomationDetails {
                detail_list_selection: _,
            }) => {
                let selected_service_bounds = selected_service_bounds.borrow();
                let automation_modes: Vec<(String, AutomationMode)> = system
                    .iter_services()
                    .dropping(service_selection.unwrap_or(0))
                    .next()
                    .map(|(service)| {
                        service
                            .definition
                            .automation
                            .iter()
                            .map(|automation_entry| {
                                let automation_name = automation_entry.name.clone();
                                // FIXME resolve properly currently mode
                                let current_mode = AutomationMode::Disabled;

                                (automation_name, current_mode)
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                let longest_name: u16 = automation_modes
                    .iter()
                    .map(|(name, _)| name.len() as u16)
                    .max()
                    .unwrap_or(0);

                render_at_pos(
                    Cell {
                        border: (active_border_color, String::from("Automation")).into(),
                        opaque: true,
                        element: List {
                            selection: 0,
                            items: automation_modes
                                .iter()
                                .map(|(name, mode)| Cell {
                                    element: Flow {
                                        direction: Dir::LeftRight,
                                        cells: vec![
                                            Cell {
                                                element: Text {
                                                    text: name.clone(),
                                                    ..Default::default()
                                                }
                                                .into_el(),
                                                min_width: longest_name + 3,
                                                padding_right: 3,
                                                ..Default::default()
                                            },
                                            Cell {
                                                element: Text {
                                                    text: match mode {
                                                        AutomationMode::Disabled => {
                                                            String::from("Disabled")
                                                        }
                                                        AutomationMode::Triggerable => {
                                                            String::from("Triggerable")
                                                        }
                                                        AutomationMode::Automatic => {
                                                            String::from("Automatic")
                                                        }
                                                    },
                                                    fg: match mode {
                                                        AutomationMode::Disabled => None,
                                                        AutomationMode::Triggerable => {
                                                            Some(secondary_active_color)
                                                        }
                                                        AutomationMode::Automatic => {
                                                            Some(active_color)
                                                        }
                                                    },
                                                    ..Default::default()
                                                }
                                                .into_el(),
                                                align_horiz: Align::End,
                                                min_width: 11,
                                                ..Default::default()
                                            },
                                        ],
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
                    // Render the floating pane after the selected service but on the same level
                    (
                        selected_service_bounds.x + selected_service_bounds.width,
                        selected_service_bounds.y,
                    ),
                    frame,
                );
            }
            _ => {}
        }
    }
}

pub enum BlockUIStatus {
    Initial,
    Disabled,
    Working,
    FailedPrerequisites,
    Failed,
    Ok,
}

fn service_list(
    system_state: &SystemState,
    profile: &Profile,
    selection: Option<usize>,
) -> (List, Rc<RefCell<Rect>>) {
    // TODO Theme?
    let active_color = Color::Rgb(0, 140, 0);
    let secondary_active_color = Color::Rgb(0, 40, 180);
    let waiting_to_process_color = Color::Rgb(230, 127, 0);
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
                // FIXME resolve from service properlycjBjj
                let show_output = true;
                let block_statuses: BTreeMap<String, BlockUIStatus> = service
                    .definition
                    .blocks
                    .iter()
                    .map(|block| {
                        (
                            block.id.clone(),
                            match service.get_block_status(&block.id) {
                                BlockStatus::Initial => BlockUIStatus::Initial,
                                BlockStatus::Working { step } => match step {
                                    WorkStep::PrerequisiteCheck { last_failure, .. }
                                        if last_failure.is_some() =>
                                    {
                                        BlockUIStatus::FailedPrerequisites
                                    }
                                    _ => BlockUIStatus::Working,
                                },
                                BlockStatus::Ok => BlockUIStatus::Ok,
                                BlockStatus::Error => BlockUIStatus::Failed,
                                BlockStatus::Disabled => BlockUIStatus::Disabled,
                            },
                        )
                    })
                    .collect();
                let is_processing = block_statuses
                    .values()
                    .any(|status| matches!(status, BlockUIStatus::Working));

                let start_elements = vec![
                    // Service name
                    Cell {
                        fill: true,
                        element: Text {
                            text: service.definition.id.to_string(),
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
                ];
                let end_elements = vec![
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
                            text: if !service.definition.automation.is_empty() {
                                "A"
                            } else {
                                "-"
                            }
                            .into(),

                            // FIXME proper color here
                            fg: inactive_color.into(),

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
                ];
                let mut block_refs: Vec<&Block> = service.definition.blocks.iter().collect();
                block_refs.sort_by_key(|block| block.status_line.slot);

                // FIXME add empty blocks if other services have blocks not listed here
                let block_elements: Vec<Cell> = block_refs
                    .into_iter()
                    .map(|block| {
                        let block_ui_status = block_statuses
                            .get(&block.id)
                            .unwrap_or(&BlockUIStatus::Failed);

                        Cell {
                            element: Text {
                                text: match block_ui_status {
                                    BlockUIStatus::Disabled => {
                                        ["-", &" ".repeat(block.status_line.symbol.len() - 1)].join("")
                                    }
                                    _ => block.status_line.symbol.clone(),
                                },
                                fg: match block_ui_status {
                                    BlockUIStatus::Initial => inactive_color.into(),
                                    BlockUIStatus::Disabled => inactive_color.into(),
                                    BlockUIStatus::FailedPrerequisites => {
                                        waiting_to_process_color.into()
                                    }
                                    BlockUIStatus::Working => processing_color.into(),
                                    BlockUIStatus::Ok => active_color.into(),
                                    BlockUIStatus::Failed => error_color.into(),
                                },
                            }
                            .into_el(),
                            ..Default::default()
                        }
                    })
                    .collect();

                Cell {
                    align_horiz: Align::Stretch,
                    store_bounds: if index == selection.unwrap_or(usize::MAX) {
                        Some(selected_service_bounds.clone())
                    } else {
                        None
                    },
                    element: Flow {
                        direction: Dir::LeftRight,
                        cells: [start_elements, block_elements, end_elements]
                            .into_iter()
                            .flatten()
                            .collect(),
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
        cells: iter::once(Cell {
            align_horiz: Align::Stretch,
            align_vert: Align::Stretch,
            fill: true,
            element: OutputDisplay {
                wrap: wrap_output,
                pos_horiz,
                lines: state
                    .output_store
                    .query_lines_to(
                        height,
                        pos_vert,
                        &get_active_outputs(&state.output_store, state),
                    )
                    .into_iter()
                    .map(|(key, line)| {
                        let color_idx = key
                            .service_id
                            .clone()
                            .and_then(|service_id| {
                                profile
                                    .services
                                    .iter()
                                    .enumerate()
                                    .find(|(_, service)| service.definition.id == service_id)
                                    .map(|(idx, _)| idx)
                            })
                            .unwrap_or(profile.services.len());
                        let name = key
                            .service_id
                            .clone()
                            .unwrap_or(profile.definition.id.clone());

                        OutputLine {
                            prefix: vec![
                                LinePart {
                                    text: match key.kind {
                                        OutputKind::System => "i/",
                                        OutputKind::ExtProcess => "c/",
                                    }
                                    .to_string(),
                                    color: match key.kind {
                                        OutputKind::System => Color::Rgb(0, 180, 0),
                                        OutputKind::ExtProcess => Color::Rgb(0, 120, 220),
                                    }
                                    .into(),
                                },
                                LinePart {
                                    text: format!("{name}/"),
                                    color: SERVICE_NAME_COLORS
                                        [color_idx % SERVICE_NAME_COLORS.len()]
                                    .into(),
                                },
                                LinePart {
                                    text: format!(
                                        "{name} | ",
                                        name = force_len(&key.source_name, 5)
                                    ),
                                    color: Some(hashed_color(&key.source_name)),
                                },
                            ],
                            parts: vec![LinePart {
                                text: line.value.clone(),
                                color: None,
                            }],
                        }
                    })
                    .collect(),
            }
            .into_el(),
            ..Default::default()
        })
        .chain(if pos_vert.is_none() {
            Some(Cell {
                align_horiz: Align::Stretch,
                element: Spinner {
                    active: true,
                    ..Default::default()
                }
                .into_el(),
                ..Default::default()
            })
        } else {
            None
        })
        .collect(),
        ..Default::default()
    }
}

fn hashed_color(name: &str) -> Color {
    // Hash the name to obtain a color for it
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    let hash: usize = hasher.finish() as usize;
    SERVICE_NAME_COLORS[hash % SERVICE_NAME_COLORS.len()]
}

fn force_len(text: &str, len: usize) -> String {
    let actual_len = text.chars().count();

    if actual_len == len {
        text.to_string()
    } else if actual_len > len {
        text.chars().take(len).collect()
    } else {
        let padding = " ".repeat(len - actual_len);
        format!("{}{}", text, padding)
    }
}
