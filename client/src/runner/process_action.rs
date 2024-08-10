use crate::config::AutoCompileMode;
use crate::models::{Action, ServiceAction, ServiceStatus};
use crate::system_state::SystemState;

fn process_action(state: &mut SystemState, action: Action) {
    match action {
        Action::Tick => {},
        Action::Shutdown => {
            state.should_exit = true;
        },
        Action::ActivateProfile(profile) => {
            state.update_state(|state| {
                state.service_statuses = profile
                    .services
                    .iter()
                    .map(|service| (service.name.clone(), ServiceStatus::from(&profile, service)))
                    .collect();
                state.current_profile = Some(profile);
            });
        },
        Action::UpdateServiceAction(service_name, action) => {
            state.update_service_status(&service_name, |status| {
                status.action = action;
            });
        },
        Action::UpdateAllServiceActions(action) => {
            state.update_all_statuses(|_, status| {
                status.action = action.clone();
            })
        },
        Action::CycleAutoCompile(service_name) => {
            state.update_service_status(&service_name, |status| {
                status.auto_compile = match status.auto_compile {
                    None => None,
                    Some(AutoCompileMode::Automatic) => Some(AutoCompileMode::Disabled),
                    Some(AutoCompileMode::Disabled) => Some(AutoCompileMode::Custom),
                    Some(AutoCompileMode::Custom) => {
                        // When changing from triggered to automatic, if there were pending changes then we should also
                        // trigger compilation
                        if status.has_uncompiled_changes {
                            status.action = ServiceAction::Recompile;
                        }
                        Some(AutoCompileMode::Automatic)
                    },
                }
            })
        },
        Action::CycleAutoCompileAll => {
            let lowest_status: AutoCompileMode = state.iter_services()
                .map(|service| {
                    state.get_service_status(&service.name()).as_ref()
                        .map(|status| status.auto_compile.as_ref())
                        .flatten()
                })
                .flatten()
                .fold(AutoCompileMode::Automatic, |left, right| {
                    match (left, right) {
                        | (AutoCompileMode::Disabled, _)
                        | (_, AutoCompileMode::Disabled) => AutoCompileMode::Disabled,
                        | (AutoCompileMode::Custom, _)
                        | (_, AutoCompileMode::Custom) => AutoCompileMode::Custom,
                        | (AutoCompileMode::Automatic, _) => AutoCompileMode::Automatic
                    }
                });
            state.update_all_statuses(|_, status| {
                status.auto_compile = status.auto_compile.as_ref().map(|_| {
                    match lowest_status {
                        AutoCompileMode::Disabled => AutoCompileMode::Custom,
                        AutoCompileMode::Custom => AutoCompileMode::Automatic,
                        AutoCompileMode::Automatic => AutoCompileMode::Disabled
                    }
                });
            })
        },
        Action::ToggleRun(service_name) => {
            state.update_service_status(&service_name, |status| {
                status.should_run = !status.should_run;
            });
        },
        Action::ToggleRunAll => {
            let new_run_state = state.iter_services()
                .map(|service| {
                    state.get_service_status(&service.name()).as_ref()
                        .map(|status| status.should_run)
                }).flatten()
                .any(|cond| !cond);

            state.update_all_statuses(|_, status| {
                status.should_run = new_run_state;
            })
        },
        Action::ToggleDebug(service_name) => {
            state.update_service_status(&service_name, |status| {
                status.debug = !status.debug;
                status.action = match status.action {
                    ServiceAction::Recompile => ServiceAction::Recompile,
                    _ => ServiceAction::Restart
                }
            });
        },
        Action::ToggleDebugAll => {
            let new_debug_state = state.iter_services()
                .map(|service| {
                    state.get_service_status(&service.name()).as_ref()
                        .map(|status| status.debug)
                }).flatten()
                .any(|cond| !cond);

            state.update_all_statuses(|_, status| {
                status.debug = new_debug_state;
                status.action = match status.action {
                    ServiceAction::Recompile => ServiceAction::Recompile,
                    _ => ServiceAction::Restart
                }
            })
        },
        Action::TriggerPendingCompiles => {
            state.update_all_statuses(|_, status| {
                if status.has_uncompiled_changes {
                    status.has_uncompiled_changes = false;
                    status.action = ServiceAction::Recompile;
                }
            })
        },
        Action::ToggleOutput(service_name) => {
            state.update_service_status(&service_name, |status| {
                status.show_output = !status.show_output;
            });
        },
        Action::ToggleOutputAll => {
            let has_disabled = state.iter_services()
                .map(|service| {
                    state.get_service_status(&service.name()).as_ref()
                        .map(|status| status.show_output)
                }).flatten()
                .any(|cond| !cond);

            state.update_all_statuses(|_, status| {
                status.show_output = has_disabled;
            })
        }
    }
}
