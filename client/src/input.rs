use std::cmp::{max, min};
use std::ops::Neg;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossterm::event::{poll as poll_events, read as read_event, Event, KeyCode, KeyModifiers};

use shared::message::models::{Profile, ServiceAction};
use shared::message::Action;
use shared::message::Action::{CycleAutoCompile, CycleAutoCompileAll, ToggleOutput, ToggleOutputAll, ToggleRun, ToggleRunAll, TriggerPendingCompiles, UpdateAllServiceActions, UpdateServiceAction};
use shared::utils::get_active_outputs;

use crate::ui::{UIState, ViewProfilePane, ViewProfileState};
use crate::{ClientState, ClientStatus};

pub fn process_inputs(client: Arc<Mutex<ClientState>>) -> Result<(), String> {
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
                    process_service_action(client, |_| UpdateAllServiceActions(ServiceAction::Restart));
                },
                KeyCode::Char('e') => {
                    process_service_action(client, |service| UpdateServiceAction(service, ServiceAction::Restart));
                },
                // Recompiling
                KeyCode::Char('c') if shift => {
                    process_service_action(client, |_| UpdateAllServiceActions(ServiceAction::Recompile));
                },
                KeyCode::Char('c') => {
                    process_service_action(client, |service| UpdateServiceAction(service, ServiceAction::Recompile));
                },
                // Cycling autocompile
                KeyCode::Char('a') if shift => {
                    process_service_action(client, |_| CycleAutoCompileAll);
                },
                KeyCode::Char('a') => {
                    process_service_action(client, |service| CycleAutoCompile(service));
                }
                // Toggling should_run
                KeyCode::Char('r') if shift => {
                    process_service_action(client, |_| ToggleRunAll);
                },
                KeyCode::Char('r') => {
                    process_service_action(client, |service| ToggleRun(service));
                }
                // Toggling output
                KeyCode::Char('o') if shift => {
                    process_service_action(client, |_| ToggleOutputAll);
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
                KeyCode::Char('d') if ctrl => {
                    let mut client = client.lock().unwrap();
                    client.status = ClientStatus::Exiting;
                }
                // Triggering pending compiles
                KeyCode::Char('t') => {
                    process_service_action(client, |_| TriggerPendingCompiles);
                }
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

fn process_navigation(client: Arc<Mutex<ClientState>>, dir: (i8, i8), boosted: bool) {
    let mut client = client.lock().unwrap();
    match &client.ui {
        | UIState::Exiting
        | UIState::Initializing => {}
        UIState::ProfileSelect { selected_idx } => {
            client.ui = UIState::ProfileSelect {
                selected_idx: update_vert_index(*selected_idx, client.config.profiles.len(), dir),
            }
        }
        UIState::ViewProfile(view_profile) => {
            let num_profiles = client
                .system_state
                .as_ref()
                .unwrap()
                .current_profile
                .as_ref()
                .unwrap()
                .services
                .len();

            match view_profile.active_pane {
                ViewProfilePane::ServiceList if dir.1 != 0 => {
                    client.ui = UIState::ViewProfile(ViewProfileState {
                        service_selection: update_vert_index(view_profile.service_selection, num_profiles, dir),
                        ..*view_profile
                    })
                }
                ViewProfilePane::OutputPane if dir.0 != 0 => {
                    client.ui = UIState::ViewProfile(ViewProfileState {
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
                    client.ui = UIState::ViewProfile(ViewProfileState {
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
                                get_active_outputs(&client.output_store, &client.system_state)
                            ).last().unwrap().1.index;
                            client.output_store.query_lines_to(
                                (dir.1.neg() as usize) * amount + if view_profile.output_pos_vert.is_none() {
                                    0
                                } else {
                                    1
                                },
                                view_profile.output_pos_vert,
                                get_active_outputs(&client.output_store, &client.system_state)
                            ).first().map(|(_, line)| max(line.index, min_index))
                        },
                        ..*view_profile
                    })
                }
                ViewProfilePane::OutputPane if dir.1 > 0 => {
                    client.ui = UIState::ViewProfile(ViewProfileState {
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
                                    get_active_outputs(&client.output_store, &client.system_state)
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

fn process_navigate_to_limit(client: Arc<Mutex<ClientState>>, limit: NavLimit) {
    let mut client = client.lock().unwrap();
    match &client.ui {
        | UIState::Exiting
        | UIState::Initializing
        | UIState::ProfileSelect { .. } => {},
        UIState::ViewProfile(view_profile) => {
            match view_profile.active_pane {
                ViewProfilePane::OutputPane => {
                    client.ui = UIState::ViewProfile(ViewProfileState {
                        output_pos_vert: {
                            match limit {
                                NavLimit::Start => Some(
                                    client.output_store.query_lines_from(
                                        client.last_frame_size.1.saturating_sub(2) as usize,
                                        None,
                                        get_active_outputs(&client.output_store, &client.system_state)
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

fn process_cycle(client: Arc<Mutex<ClientState>>) {
    let mut client = client.lock().unwrap();
    match &client.ui {
        | UIState::Initializing
        | UIState::Exiting
        | UIState::ProfileSelect { .. } => {}
        UIState::ViewProfile(view_profile) => {
            match view_profile.active_pane {
                ViewProfilePane::ServiceList => {
                    client.ui = UIState::ViewProfile(ViewProfileState {
                        active_pane: ViewProfilePane::OutputPane,
                        ..*view_profile
                    })
                }
                ViewProfilePane::OutputPane => {
                    client.ui = UIState::ViewProfile(ViewProfileState {
                        active_pane: ViewProfilePane::ServiceList,
                        ..*view_profile
                    })
                }
            }
        }
    }
}

fn process_select(client: Arc<Mutex<ClientState>>) {
    let mut client = client.lock().unwrap();

    match client.ui {
        | UIState::Exiting
        | UIState::Initializing => {}
        UIState::ProfileSelect { selected_idx } => {
            let selection = client.config.profiles.get(selected_idx);

            if let Some(profile) = selection {
                let action =
                    Action::ActivateProfile(Profile::new(profile, &client.config.services));
                client.actions_out.push_back(action);
            }
        }
        UIState::ViewProfile { .. } => {
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

fn process_service_action<F>(
    client: Arc<Mutex<ClientState>>,
    create_action: F
) where F: Fn(String) -> Action {
    let mut client = client.lock().unwrap();

    match &client.ui {
        UIState::ViewProfile(view_profile)
        if matches!(view_profile.active_pane, ViewProfilePane::ServiceList) => {
            let service_name = client
                .system_state.as_ref().unwrap()
                .current_profile.as_ref().unwrap()
                .services[view_profile.service_selection]
                .name
                .clone();
            client.actions_out.push_back(create_action(service_name));
        }
        _ => {}
    }
}

fn process_toggle_output_wrap(client: Arc<Mutex<ClientState>>) {
    let mut client = client.lock().unwrap();

    match &client.ui {
        UIState::ViewProfile(view_profile) => {
            client.ui = UIState::ViewProfile(ViewProfileState {
                wrap_output: !view_profile.wrap_output,
                output_pos_horiz: None,
                ..*view_profile
            })
        }
        _ => {}
    }
}