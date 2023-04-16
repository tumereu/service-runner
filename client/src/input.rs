use std::cmp::min;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossterm::event::{Event, KeyCode, poll as poll_events, read as read_event};

use shared::message::Action;
use shared::message::models::Profile;

use crate::{ClientState, ClientStatus};
use crate::ui::{UIState, ViewProfilePane};

pub fn process_inputs(state: Arc<Mutex<ClientState>>) -> Result<(), String> {
    let config = state.lock().unwrap().config.clone();

    if poll_events(Duration::from_millis(0)).unwrap() {
        let _port = config.server.port;
        let event = read_event().unwrap();

        if let Event::Key(key) = event {
            match key.code {
                // Controls to exit
                KeyCode::Esc => {
                    state.lock().unwrap().status = ClientStatus::Exiting;
                },
                // Generic navigation controls
                KeyCode::Left | KeyCode::Char('h') => process_navigation(state, (-1, 0)),
                KeyCode::Right | KeyCode::Char('l') => process_navigation(state, (1, 0)),
                KeyCode::Up | KeyCode::Char('k') => process_navigation(state, (0, -1)),
                KeyCode::Down | KeyCode::Char('j') => process_navigation(state, (0, 1)),
                // Generic selection controls
                KeyCode::Enter | KeyCode::Char(' ') => process_select(state),
                // Disregard everything else
                _ => {}
            }
        }
    }

    Ok(())
}

fn process_navigation(state: Arc<Mutex<ClientState>>, dir: (i8, i8)) {
    let mut state = state.lock().unwrap();
    match &state.ui {
        UIState::Initializing => {},
        UIState::ProfileSelect { selected_idx } => {
            state.ui = UIState::ProfileSelect {
                selected_idx: update_vert_index(*selected_idx, state.config.profiles.len(), dir)
            }
        }
        UIState::ViewProfile { active_pane, service_selection } => {
            let num_profiles = state.system_state.as_ref().unwrap().current_profile.as_ref().unwrap().services.len();

            match active_pane {
                ViewProfilePane::ServiceList if dir.0 > 0 => {
                    state.ui = UIState::ViewProfile {
                        active_pane: ViewProfilePane::OutputPane,
                        service_selection: *service_selection
                    }
                },
                ViewProfilePane::ServiceList if dir.1 != 0 => {
                    state.ui = UIState::ViewProfile {
                        active_pane: ViewProfilePane::ServiceList,
                        service_selection: update_vert_index(*service_selection, num_profiles, dir)
                    }
                }
                ViewProfilePane::OutputPane if dir.0 < 0 => {
                    state.ui = UIState::ViewProfile {
                        service_selection: *service_selection,
                        active_pane: ViewProfilePane::ServiceList
                    }
                },
                _ => {}
            }
        }
    }
}

fn process_select(state: Arc<Mutex<ClientState>>) {
    let mut state = state.lock().unwrap();

    match state.ui {
        UIState::Initializing => {},
        UIState::ProfileSelect { selected_idx } => {
            let selection = state.config.profiles.get(selected_idx);


            if let Some(profile) = selection {
                let action = Action::ActivateProfile(Profile::new(
                    profile,
                    &state.config.services
                ));
                state.actions_out.push(action);
            }
        }
        UIState::ViewProfile { .. } => {
            // TODO
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