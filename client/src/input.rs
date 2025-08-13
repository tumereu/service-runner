use std::cmp::{max, min};
use std::ops::Neg;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use crate::config::TaskDefinitionId;
use crate::models::{get_active_outputs, Action, Action::*, BlockAction, Profile};
use crate::runner::process_action::process_action;
use crate::ui::{CurrentScreen, ViewProfileFloatingPane, ViewProfilePane, ViewProfileState};
use crate::SystemState;
use crossterm::event::{poll as poll_events, read as read_event, Event, KeyCode, KeyModifiers};
use log::debug;

fn toggle_automation_detailed_controls(system_arc: Arc<RwLock<SystemState>>, _all: bool) {
    let mut system = system_arc.write().unwrap();
    match &mut system.ui.screen {
        CurrentScreen::ViewProfile(view_profile) if view_profile.active_pane == ViewProfilePane::ServiceList => {
            match view_profile.floating_pane {
                Some(ViewProfileFloatingPane::ServiceAutomationDetails { .. }) => view_profile.floating_pane = None,
                _ => view_profile.floating_pane = ViewProfileFloatingPane::ServiceAutomationDetails {
                    detail_list_selection: 0
                }.into()
            }
        },
        _ => {}
    }
}

fn process_navigation(system_arc: Arc<RwLock<SystemState>>, dir: (i8, i8), boosted: bool) {
    let mut system = system_arc.write().unwrap();

    match &system.ui.screen {
        CurrentScreen::ProfileSelect { selected_idx } => {
            system.ui.screen = CurrentScreen::ProfileSelect {
                selected_idx: update_vert_index(*selected_idx, system.config.profiles.len(), dir),
            }
        }
        CurrentScreen::ViewProfile(view_profile) => {
            let num_profiles = system
                .current_profile
                .as_ref()
                .unwrap()
                .services
                .len();

            match view_profile.active_pane {
                ViewProfilePane::ServiceList if dir.1 != 0 => {
                    system.ui.screen = CurrentScreen::ViewProfile(ViewProfileState {
                        service_selection: update_vert_index(view_profile.service_selection, num_profiles, dir),
                        ..*view_profile
                    })
                }
                ViewProfilePane::OutputPane if dir.0 != 0 => {
                    system.ui.screen = CurrentScreen::ViewProfile(ViewProfileState {
                        output_pos_horiz: {
                            let amount: u64 = if boosted {
                                64
                            } else {
                                1
                            };
                            Some(
                                if dir.0 > 0 {
                                    view_profile.output_pos_horiz.unwrap_or(0).saturating_add(amount)
                                } else {
                                    view_profile.output_pos_horiz.unwrap_or(0).saturating_sub(amount)
                                }
                            )
                        },
                        ..*view_profile
                    })
                }
                ViewProfilePane::OutputPane if dir.1 < 0 => {
                    system.ui.screen = CurrentScreen::ViewProfile(ViewProfileState {
                        output_pos_vert: {
                            let amount: usize = if boosted {
                                (system.ui.last_frame_size.1 / 2) as usize
                            } else {
                                1
                            };
                            let active_outputs = get_active_outputs(&system.output_store, &system);
                            // Prevent users from scrolling past the first line of output
                            let min_index = system.output_store.query_lines_from(
                                system.ui.last_frame_size.1.saturating_sub(2) as usize,
                                None,
                                &active_outputs,
                            ).last().unwrap().1.index;

                            system.output_store.query_lines_to(
                                (dir.1.neg() as usize) * amount + if view_profile.output_pos_vert.is_none() {
                                    0
                                } else {
                                    1
                                },
                                view_profile.output_pos_vert,
                                &active_outputs,
                            ).first().map(|(_, line)| max(line.index, min_index))
                        },
                        ..*view_profile
                    })
                }
                ViewProfilePane::OutputPane if dir.1 > 0 => {
                    system.ui.screen = CurrentScreen::ViewProfile(ViewProfileState {
                        output_pos_vert: {
                            if view_profile.output_pos_vert.is_some() {
                                let amount: usize = if boosted {
                                    (system.ui.last_frame_size.1 / 2) as usize
                                } else {
                                    1
                                };
                                let lines = system.output_store.query_lines_from(
                                    (dir.1 as usize) * amount + 1,
                                    view_profile.output_pos_vert,
                                    &get_active_outputs(&system.output_store, &system)
                                );
                                if lines.len() == (dir.1 as usize) * amount + 1 {
                                    lines.last().map(|(_, line)| line.index)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        },
                        ..*view_profile
                    })
                }
                _ => {}
            }
        }
    }
}

enum NavLimit {
    End, Start
}

fn process_navigate_to_limit(system_arc: Arc<RwLock<SystemState>>, limit: NavLimit) {
    let mut system = system_arc.read().unwrap();
    
    /*

    match &system.ui.screen {
        CurrentScreen::ProfileSelect { .. } => {},
        CurrentScreen::ViewProfile(view_profile) => {
            match view_profile.active_pane {
                ViewProfilePane::OutputPane => {
                    system.ui.screen = CurrentScreen::ViewProfile(ViewProfileState {
                        output_pos_vert: {
                            match limit {
                                NavLimit::Start => Some(
                                    system.output_store.query_lines_from(
                                        system.ui.last_frame_size.1.saturating_sub(2) as usize,
                                        None,
                                        &get_active_outputs(&system.output_store, &system)
                                    ).last().unwrap().1.index
                                ),
                                NavLimit::End => None
                            }
                        },
                        ..*view_profile
                    })
                }
                _ => {}
            }
        }
    }
    
     */
}

fn update_vert_index(current: usize, list_len: usize, dir: (i8, i8)) -> usize {
    if dir.1 < 0 {
        current.saturating_sub(1)
    } else if dir.1 > 0 {
        min(list_len.saturating_sub(1), current.saturating_add(1))
    } else {
        current
    }
}