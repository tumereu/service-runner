use std::time::Instant;
use crate::models::{Action, ServiceAction, ServiceStatus, PendingAutomation};
use crate::models::AutomationMode::{Automatic, Disabled};
use crate::runner::automation::process_pending_automations;
use crate::system_state::SystemState;
use crate::ui::CurrentScreen;

pub fn process_action(system: &mut SystemState, action: Action) {
    match action {
        Action::Tick => {},
        Action::Shutdown => {
            system.should_exit = true;
        },
        Action::ActivateProfile(profile) => {
            system.update_state(|state| {
                state.service_statuses = profile
                    .services
                    .iter()
                    .map(|service| (service.name.clone(), ServiceStatus::from(&profile, service)))
                    .collect();
                state.current_profile = Some(profile);
                state.ui.screen = CurrentScreen::view_profile();
            });
        },
        Action::Recompile(service_name) => {
            system.update_service_status(&service_name, |status| {
                status.action = ServiceAction::Recompile;
            });
        },
        Action::Restart(service_name) => {
            system.update_service_status(&service_name, |status| {
                status.action = ServiceAction::Restart;
            });
        },
        Action::RecompileAll => {
            system.update_all_statuses(|_, status| {
                status.action = ServiceAction::Recompile;
            })
        },
        Action::RestartAll => {
            system.update_all_statuses(|_, status| {
                status.action = ServiceAction::Restart;
            })
        },
        Action::CycleAutomation(service_name) => {
            system.update_service_status(&service_name, |status| {
                // TODO maybe toggle a separate "automation disabled" instead.
                if status.automation_modes.iter().all(|(_, mode)| *mode == Disabled) {
                    status.automation_modes = status.automation_modes.iter()
                        .map(|(key, _)| (key.clone(), Automatic))
                        .collect()
                } else {
                    status.automation_modes = status.automation_modes.iter()
                        .map(|(key, _)| (key.clone(), Disabled))
                        .collect()
                }
            })
        },
        Action::UpdateRun(service_name, should_run) => {
            system.update_service_status(&service_name, |status| {
                status.should_run = should_run;
            });
        }
        Action::ToggleRun(service_name) => {
            system.update_service_status(&service_name, |status| {
                status.should_run = !status.should_run;
            });
        },
        Action::ToggleRunAll => {
            let new_run_state = system.iter_services()
                .map(|service| {
                    system.get_service_status(&service.name).as_ref()
                        .map(|status| status.should_run)
                }).flatten()
                .any(|cond| !cond);

            system.update_all_statuses(|_, status| {
                status.should_run = new_run_state;
            })
        },
        Action::ToggleDebug(service_name) => {
            system.update_service_status(&service_name, |status| {
                status.debug = !status.debug;
                status.action = match status.action {
                    ServiceAction::Recompile => ServiceAction::Recompile,
                    _ => ServiceAction::Restart
                }
            });
        },
        Action::ToggleDebugAll => {
            let new_debug_state = system.iter_services()
                .map(|service| {
                    system.get_service_status(&service.name).as_ref()
                        .map(|status| status.debug)
                }).flatten()
                .any(|cond| !cond);

            system.update_all_statuses(|_, status| {
                status.debug = new_debug_state;
                status.action = match status.action {
                    ServiceAction::Recompile => ServiceAction::Recompile,
                    _ => ServiceAction::Restart
                }
            })
        },
        Action::TriggerPendingAutomations => {
            system.update_all_statuses(|_, status| {
                status.pending_automations = status.pending_automations.iter()
                    .map(|pending_automation| {
                        PendingAutomation {
                            not_before: Instant::now(),
                            effect: pending_automation.effect
                        }
                    }).collect();
            });
            process_pending_automations(system);
        }
        Action::ToggleOutput(service_name) => {
            system.update_service_status(&service_name, |status| {
                status.show_output = !status.show_output;
            });
        },
        Action::ToggleOutputAll => {
            let has_disabled = system.iter_services()
                .map(|service| {
                    system.get_service_status(&service.name).as_ref()
                        .map(|status| status.show_output)
                }).flatten()
                .any(|cond| !cond);

            system.update_all_statuses(|_, status| {
                status.show_output = has_disabled;
            })
        },
        Action::Reset(service_name) => {
            // TODO implement
        },
        Action::ResetAll => {
            // TODO implement
        }
    }
}
