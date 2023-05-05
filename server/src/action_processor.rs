use std::os::linux::raw::stat;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use itertools::Itertools;
use shared::config::AutoCompileMode::AUTOMATIC;

use shared::message::models::{AutoCompileMode, ServiceAction, ServiceStatus};
use shared::message::Action;
use shared::system_state::Status;

use crate::server_state::ServerState;

pub fn start_action_processor(server: Arc<Mutex<ServerState>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while server.lock().unwrap().get_state().status != Status::Exiting {
            {
                let mut server = server.lock().unwrap();
                while let Some(action) = server.actions_in.pop_front() {
                    process_action(&mut server, action);
                }
            }

            thread::sleep(Duration::from_millis(10))
        }
    })
}

fn process_action(server: &mut ServerState, action: Action) {
    match action {
        Action::Tick => {},
        Action::Shutdown => {
            server.update_state(|state| {
                state.status = Status::Exiting;
            });
        },
        Action::ActivateProfile(profile) => {
            server.update_state(|state| {
                state.service_statuses = profile
                    .services
                    .iter()
                    .map(|service| (service.name.clone(), ServiceStatus::from(&profile, service)))
                    .collect();
                state.current_profile = Some(profile);
            });
        },
        Action::UpdateServiceAction(service_name, action) => {
            server.update_service_status(&service_name, |status| {
                status.action = action;
            });
        },
        Action::UpdateAllServiceActions(action) => {
            server.update_all_statuses(|_, status| {
                status.action = action.clone();
            })
        },
        Action::CycleAutoCompile(service_name) => {
            server.update_service_status(&service_name, |status| {
                status.auto_compile = match status.auto_compile {
                    None => None,
                    Some(AutoCompileMode::AUTOMATIC) => Some(AutoCompileMode::DISABLED),
                    Some(AutoCompileMode::DISABLED) => Some(AutoCompileMode::TRIGGERED),
                    Some(AutoCompileMode::TRIGGERED) => {
                        // When changing from triggered to automatic, if there were pending changes then we should also
                        // trigger compilation
                        if status.has_uncompiled_changes {
                            status.action = ServiceAction::Recompile;
                        }
                        Some(AutoCompileMode::AUTOMATIC)
                    },
                }
            })
        },
        Action::CycleAutoCompileAll => {
            let lowest_status: AutoCompileMode = server.iter_services()
                .map(|service| {
                    server.get_service_status(&service.name).as_ref()
                        .map(|status| status.auto_compile.as_ref())
                        .flatten()
                })
                .flatten()
                .fold(AutoCompileMode::AUTOMATIC, |left, right| {
                    match (left, right) {
                        | (AutoCompileMode::DISABLED, _)
                        | (_, AutoCompileMode::DISABLED) => AutoCompileMode::DISABLED,
                        | (AutoCompileMode::TRIGGERED, _)
                        | (_, AutoCompileMode::TRIGGERED) => AutoCompileMode::TRIGGERED,
                        | (AutoCompileMode::AUTOMATIC, _)
                        | (_, AutoCompileMode::AUTOMATIC) => AutoCompileMode::AUTOMATIC
                    }
                });
            server.update_all_statuses(|_, status| {
                status.auto_compile = status.auto_compile.as_ref().map(|_| {
                    match lowest_status {
                        AutoCompileMode::DISABLED => AutoCompileMode::TRIGGERED,
                        AutoCompileMode::TRIGGERED => AutoCompileMode::AUTOMATIC,
                        AutoCompileMode::AUTOMATIC => AutoCompileMode::DISABLED
                    }
                });
            })
        },
        Action::ToggleRun(service_name) => {
            server.update_service_status(&service_name, |status| {
                status.should_run = !status.should_run;
            });
        },
        Action::ToggleRunAll => {
            let has_non_running = server.iter_services()
                .map(|service| {
                    server.get_service_status(&service.name).as_ref()
                        .map(|status| status.should_run)
                }).flatten()
                .any(|cond| !cond);

            server.update_all_statuses(|_, status| {
                status.should_run = has_non_running;
            })
        }
    }
}
