use std::cmp::{max, min};
use std::ops::Neg;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::models::{get_active_outputs, Action, Action::*, BlockAction, Profile};
use crate::runner::process_action::process_action;
use crate::ui::{CurrentScreen, ViewProfileFloatingPane, ViewProfilePane, ViewProfileState};
use crate::SystemState;
use crossterm::event::{poll as poll_events, read as read_event, Event, KeyCode, KeyModifiers};
use log::debug;

pub fn process_inputs(system_arc: Arc<Mutex<SystemState>>) {
    while poll_events(Duration::from_millis(0)).unwrap_or(false) {
        let system = system_arc.clone();
        let event = read_event().unwrap();

        if let Event::Key(key) = event {
            debug!("Received input event {key:?}");

            let shift = key.modifiers.contains(KeyModifiers::SHIFT);
            let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

            let code = match key.code {
                KeyCode::Char(character) => KeyCode::Char(character.to_ascii_lowercase()),
                any => any
            };

            let selected_service_id = {
                let state = system_arc.lock().unwrap();
                match &state.ui.screen {
                    CurrentScreen::ViewProfile(view_profile) => {
                        state.current_profile.as_ref()
                            .and_then(|profile| {
                                profile.services.get(view_profile.service_selection)
                                    .map(|service| service.definition.id.clone())
                            })
                    },
                    _ => None
                }
            };

            match code {
                // Generic navigation controls
                KeyCode::Left | KeyCode::Char('h') => process_navigation(system, (-1, 0), shift || ctrl),
                KeyCode::Right | KeyCode::Char('l') => process_navigation(system, (1, 0), shift || ctrl),
                KeyCode::Up | KeyCode::Char('k') => process_navigation(system, (0, -1), shift || ctrl),
                KeyCode::Down | KeyCode::Char('j') => process_navigation(system, (0, 1), shift || ctrl),
                KeyCode::Tab => process_cycle(system),
                // Generic selection controls
                KeyCode::Enter | KeyCode::Char(' ') => process_select(system),
                // Output wrapping controls
                KeyCode::Char('w') => process_toggle_output_wrap(system),
                // Service interaction specific controls
                // FIXME extract into a control scheme mapping, block names are hardcoded atm
                // Restarting
                KeyCode::Char('e') => {
                    process_service_action(
                        system,
                        if shift { None } else { selected_service_id },
                        "run",
                        BlockAction::ReRun,
                    );
                },
                // Recompiling
                KeyCode::Char('c') => {
                    process_service_action(
                        system,
                        if shift { None } else { selected_service_id },
                        "build",
                        BlockAction::ReRun,
                    );
                },

                // Controlling automation
                KeyCode::Char('a') if shift => {
                    process_global_action(system, ToggleAutomationAll);
                },
                KeyCode::Char('a') if ctrl => {
                    toggle_automation_detailed_controls(system, false);
                },
                KeyCode::Char('a') => {
                    // FIXME toggle automation
                }

                // Toggling should-run
                KeyCode::Char('r') => {
                    process_service_action(
                        system,
                        if shift { None } else { selected_service_id },
                        "run",
                        BlockAction::ToggleEnabled,
                    );
                }
                // Toggling output
                KeyCode::Char('o') if shift => {
                    process_global_action(system, ToggleOutputAll);
                },
                // Toggling output
                KeyCode::Char('o') => {
                    // FIXME toggle output
                },
                // Controls to exit
                KeyCode::Char('q') if ctrl => {
                    process_action(&mut system.lock().unwrap(), Shutdown);
                }
                // Triggering pending compiles. This can be used even if the focus is on the output window
                KeyCode::Char('t') => process_global_action(system, TriggerPendingAutomations),
                // Scroll to start/end of the output
                KeyCode::Char('g') => {
                    process_navigate_to_limit(
                        system,
                        if shift {
                            NavLimit::End
                        } else {
                            NavLimit::Start
                        }
                    );
                }
                // Disregard everything else
                _ => {}
            }
        }
    }
}

fn toggle_automation_detailed_controls(system_arc: Arc<Mutex<SystemState>>, _all: bool) {
    let mut system = system_arc.lock().unwrap();
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

fn process_navigation(system_arc: Arc<Mutex<SystemState>>, dir: (i8, i8), boosted: bool) {
    let mut system = system_arc.lock().unwrap();

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

fn process_navigate_to_limit(system_arc: Arc<Mutex<SystemState>>, limit: NavLimit) {
    let mut system = system_arc.lock().unwrap();

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
}

fn process_cycle(system_arc: Arc<Mutex<SystemState>>) {
    let mut system = system_arc.lock().unwrap();
    match &system.ui.screen {
        CurrentScreen::ProfileSelect { .. } => {}
        CurrentScreen::ViewProfile(view_profile) => {
            match view_profile.active_pane {
                ViewProfilePane::ServiceList => {
                    system.ui.screen = CurrentScreen::ViewProfile(ViewProfileState {
                        active_pane: ViewProfilePane::OutputPane,
                        ..*view_profile
                    })
                }
                ViewProfilePane::OutputPane => {
                    system.ui.screen = CurrentScreen::ViewProfile(ViewProfileState {
                        active_pane: ViewProfilePane::ServiceList,
                        ..*view_profile
                    })
                }
            }
        }
    }
}

fn process_select(system_arc: Arc<Mutex<SystemState>>) {
    let mut system = system_arc.lock().unwrap();

    match system.ui.screen {
        CurrentScreen::ProfileSelect { selected_idx } => {
            let selection = system.config.profiles.get(selected_idx);

            if let Some(profile) = selection {
                let action = ActivateProfile(Profile::new(profile.clone(), &system.config.services));
                process_action(&mut system, action);
            }
        }
        CurrentScreen::ViewProfile { .. } => {
            // TODO change UI state so that we show a dialog or something with options?
        }
    }
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

fn process_global_action(
    system_arc: Arc<Mutex<SystemState>>,
    action: Action
) {
    let mut system = system_arc.lock().unwrap();

    match &system.ui.screen {
        CurrentScreen::ViewProfile(_) => {
            process_action(&mut system, action);
        },
        _ => {}
    }
}

fn process_service_action(
    system_arc: Arc<Mutex<SystemState>>,
    target_service: Option<String>,
    block_id: &str,
    action: BlockAction,
) {
    debug!("Processing service action {action:?} for target={target_service:?}.{block_id}");
    let mut system = system_arc.lock().unwrap();

    match &system.ui.screen {
        CurrentScreen::ViewProfile(view_profile)
        if matches!(view_profile.active_pane, ViewProfilePane::ServiceList) => {
            let affected_services: Vec<String> = system.iter_services()
                .filter(|service| match &target_service {
                    None => true,
                    Some(target_service_id) => target_service_id == &service.definition.id,
                }).map(|service| service.definition.id.clone())
                .collect();

            affected_services.iter().for_each(|service_id| {
                system.update_service(&service_id, |service| {
                    service.update_block_action(&block_id, Some(action.clone()));
                });
            });
        }
        _ => {}
    }
}

fn process_toggle_output_wrap(client: Arc<Mutex<SystemState>>) {
    let mut client = client.lock().unwrap();

    match &client.ui.screen {
        CurrentScreen::ViewProfile(view_profile) => {
            client.ui.screen = CurrentScreen::ViewProfile(ViewProfileState {
                wrap_output: !view_profile.wrap_output,
                output_pos_horiz: None,
                ..*view_profile
            })
        }
        _ => {}
    }
}