use std::cmp::{max, min};
use std::ops::Neg;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossterm::event::{poll as poll_events, read as read_event, Event, KeyCode, KeyModifiers};

use crate::models::action::models::{get_active_outputs, Profile, ServiceAction};
use crate::models::action::Action;
use crate::models::action::Action::{CycleAutoCompile, CycleAutoCompileAll, ToggleDebug, ToggleDebugAll, ToggleOutput, ToggleOutputAll, ToggleRun, ToggleRunAll, TriggerPendingCompiles, UpdateAllServiceActions, UpdateServiceAction};

use crate::ui::{CurrentScreen, ViewProfileFloatingPane, ViewProfilePane, ViewProfileState};
use crate::{SystemState, ClientStatus};

pub fn process_inputs(client: Arc<Mutex<SystemState>>) -> Result<(), String> {
    while poll_events(Duration::from_millis(0)).unwrap_or(false) {
        let client = client.clone();
        let event = read_event().unwrap();

        if let Event::Key(key) = event {
            let shift = key.modifiers.contains(KeyModifiers::SHIFT);
            let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

            let code = match key.code {
                KeyCode::Char(character) => KeyCode::Char(character.to_ascii_lowercase()),
                any @ _ => any
            };

            match code {
                // Generic navigation controls
                KeyCode::Left | KeyCode::Char('h') => process_navigation(client, (-1, 0), shift || ctrl),
                KeyCode::Right | KeyCode::Char('l') => process_navigation(client, (1, 0), shift || ctrl),
                KeyCode::Up | KeyCode::Char('k') => process_navigation(client, (0, -1), shift || ctrl),
                KeyCode::Down | KeyCode::Char('j') => process_navigation(client, (0, 1), shift || ctrl),
                KeyCode::Tab => process_cycle(client),
                // Generic selection controls
                KeyCode::Enter | KeyCode::Char(' ') => process_select(client),
                // Output wrapping controls
                KeyCode::Char('w') => process_toggle_output_wrap(client),
                // Service interaction specific controls
                // Restarting
                KeyCode::Char('e') if shift => {
                    process_global_action(client, UpdateAllServiceActions(ServiceAction::Restart));
                },
                KeyCode::Char('e') => {
                    process_service_action(client, |service| UpdateServiceAction(service, ServiceAction::Restart));
                },
                // Recompiling
                KeyCode::Char('c') if shift => {
                    process_global_action(client, UpdateAllServiceActions(ServiceAction::Recompile));
                },
                KeyCode::Char('c') => {
                    process_service_action(client, |service| UpdateServiceAction(service, ServiceAction::Recompile));
                },

                // Controlling autocompile
                KeyCode::Char('a') if shift && ctrl => {
                    process_autocomplete_details(client, true);
                },
                KeyCode::Char('a') if ctrl => {
                    process_autocomplete_details(client, false);
                },
                KeyCode::Char('a') if shift => {
                    process_global_action(client, CycleAutoCompileAll);
                },
                KeyCode::Char('a') => {
                    process_service_action(client, |service| CycleAutoCompile(service));
                }

                // Toggling should-run
                KeyCode::Char('r') if shift => {
                    process_global_action(client, ToggleRunAll);
                },
                KeyCode::Char('r') => {
                    process_service_action(client, |service| ToggleRun(service));
                }
                // Detaching from a running service?
                KeyCode::Char('d') if ctrl && !shift => {
                    let mut client = client.lock().unwrap();
                    client.status = ClientStatus::Exiting;
                }
                // Toggling debugging
                KeyCode::Char('d') if shift => {
                    process_global_action(client, ToggleDebugAll);
                },
                KeyCode::Char('d') => {
                    process_service_action(client, |service| ToggleDebug(service));
                }
                // Toggling output
                KeyCode::Char('o') if shift => {
                    process_global_action(client, ToggleOutputAll);
                },
                // Toggling output
                KeyCode::Char('o') => {
                    process_service_action(client, |service| ToggleOutput(service));
                },
                // Controls to exit
                KeyCode::Char('q') if ctrl => {
                    let mut client = client.lock().unwrap();
                    client.actions_out.push_back(Action::Shutdown);
                }
                // Triggering pending compiles. This can be used even if the focus is on the output window
                KeyCode::Char('t') => process_global_action(client, TriggerPendingCompiles),
                // Scroll to start/end of the output
                KeyCode::Char('g') => {
                    process_navigate_to_limit(
                        client,
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

    Ok(())
}

fn process_autocomplete_details(client: Arc<Mutex<SystemState>>, all: bool) {
    let mut client = client.lock().unwrap();
    match &mut client.ui {
        CurrentScreen::ViewProfile(view_profile) if view_profile.active_pane == ViewProfilePane::ServiceList => {
            match view_profile.floating_pane {
                Some(ViewProfileFloatingPane::ServiceAutocompleteDetails { .. }) => view_profile.floating_pane = None,
                _ => view_profile.floating_pane = ViewProfileFloatingPane::ServiceAutocompleteDetails {
                    detail_list_selection: 0
                }.into()
            }
        },
        _ => {}
    }
}

fn process_navigation(client: Arc<Mutex<SystemState>>, dir: (i8, i8), boosted: bool) {
    let mut client = client.lock().unwrap();
    match &client.ui {
        | CurrentScreen::Exiting
        | CurrentScreen::Initializing => {}
        CurrentScreen::ProfileSelect { selected_idx } => {
            client.ui = CurrentScreen::ProfileSelect {
                selected_idx: update_vert_index(*selected_idx, client.config.profiles.len(), dir),
            }
        }
        CurrentScreen::ViewProfile(view_profile) => {
            let num_profiles = client
                .runner_state
                .as_ref()
                .unwrap()
                .current_profile
                .as_ref()
                .unwrap()
                .services
                .len();

            match view_profile.active_pane {
                ViewProfilePane::ServiceList if dir.1 != 0 => {
                    client.ui = CurrentScreen::ViewProfile(ViewProfileState {
                        service_selection: update_vert_index(view_profile.service_selection, num_profiles, dir),
                        ..*view_profile
                    })
                }
                ViewProfilePane::OutputPane if dir.0 != 0 => {
                    client.ui = CurrentScreen::ViewProfile(ViewProfileState {
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
                    client.ui = CurrentScreen::ViewProfile(ViewProfileState {
                        output_pos_vert: {
                            let amount: usize = if boosted {
                                (client.last_frame_size.1 / 2) as usize
                            } else {
                                1
                            };
                            // Prevent users from scrolling past the first line of output
                            let min_index = client.output_store.query_lines_from(
                                client.last_frame_size.1.saturating_sub(2) as usize,
                                None,
                                get_active_outputs(&client.output_store, &client.runner_state)
                            ).last().unwrap().1.index;
                            client.output_store.query_lines_to(
                                (dir.1.neg() as usize) * amount + if view_profile.output_pos_vert.is_none() {
                                    0
                                } else {
                                    1
                                },
                                view_profile.output_pos_vert,
                                get_active_outputs(&client.output_store, &client.runner_state)
                            ).first().map(|(_, line)| max(line.index, min_index))
                        },
                        ..*view_profile
                    })
                }
                ViewProfilePane::OutputPane if dir.1 > 0 => {
                    client.ui = CurrentScreen::ViewProfile(ViewProfileState {
                        output_pos_vert: {
                            if view_profile.output_pos_vert.is_some() {
                                let amount: usize = if boosted {
                                    (client.last_frame_size.1 / 2) as usize
                                } else {
                                    1
                                };
                                let lines = client.output_store.query_lines_from(
                                    (dir.1 as usize) * amount + 1,
                                    view_profile.output_pos_vert,
                                    get_active_outputs(&client.output_store, &client.runner_state)
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

fn process_navigate_to_limit(client: Arc<Mutex<SystemState>>, limit: NavLimit) {
    let mut client = client.lock().unwrap();
    match &client.ui {
        | CurrentScreen::Exiting
        | CurrentScreen::Initializing
        | CurrentScreen::ProfileSelect { .. } => {},
        CurrentScreen::ViewProfile(view_profile) => {
            match view_profile.active_pane {
                ViewProfilePane::OutputPane => {
                    client.ui = CurrentScreen::ViewProfile(ViewProfileState {
                        output_pos_vert: {
                            match limit {
                                NavLimit::Start => Some(
                                    client.output_store.query_lines_from(
                                        client.last_frame_size.1.saturating_sub(2) as usize,
                                        None,
                                        get_active_outputs(&client.output_store, &client.runner_state)
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

fn process_cycle(client: Arc<Mutex<SystemState>>) {
    let mut client = client.lock().unwrap();
    match &client.ui {
        | CurrentScreen::Initializing
        | CurrentScreen::Exiting
        | CurrentScreen::ProfileSelect { .. } => {}
        CurrentScreen::ViewProfile(view_profile) => {
            match view_profile.active_pane {
                ViewProfilePane::ServiceList => {
                    client.ui = CurrentScreen::ViewProfile(ViewProfileState {
                        active_pane: ViewProfilePane::OutputPane,
                        ..*view_profile
                    })
                }
                ViewProfilePane::OutputPane => {
                    client.ui = CurrentScreen::ViewProfile(ViewProfileState {
                        active_pane: ViewProfilePane::ServiceList,
                        ..*view_profile
                    })
                }
            }
        }
    }
}

fn process_select(client: Arc<Mutex<SystemState>>) {
    let mut client = client.lock().unwrap();

    match client.ui {
        | CurrentScreen::Exiting
        | CurrentScreen::Initializing => {}
        CurrentScreen::ProfileSelect { selected_idx } => {
            let selection = client.config.profiles.get(selected_idx);

            if let Some(profile) = selection {
                let action =
                    Action::ActivateProfile(Profile::new(profile, &client.config.services));
                client.actions_out.push_back(action);
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
    client: Arc<Mutex<SystemState>>,
    action: Action
) {
    let mut client = client.lock().unwrap();

    match &client.ui {
        CurrentScreen::ViewProfile(view_profile) => {
            client.actions_out.push_back(action);
        },
        _ => {}
    }
}

fn process_service_action<F>(
    client: Arc<Mutex<SystemState>>,
    create_action: F
) where F: Fn(String) -> Action {
    let mut client = client.lock().unwrap();

    match &client.ui {
        CurrentScreen::ViewProfile(view_profile)
        if matches!(view_profile.active_pane, ViewProfilePane::ServiceList) => {
            let service_name = client
                .runner_state.as_ref().unwrap()
                .current_profile.as_ref().unwrap()
                .services[view_profile.service_selection]
                .name
                .clone();
            client.actions_out.push_back(create_action(service_name));
        }
        _ => {}
    }
}

fn process_toggle_output_wrap(client: Arc<Mutex<SystemState>>) {
    let mut client = client.lock().unwrap();

    match &client.ui {
        CurrentScreen::ViewProfile(view_profile) => {
            client.ui = CurrentScreen::ViewProfile(ViewProfileState {
                wrap_output: !view_profile.wrap_output,
                output_pos_horiz: None,
                ..*view_profile
            })
        }
        _ => {}
    }
}