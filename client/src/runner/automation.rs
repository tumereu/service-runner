use std::ops::Add;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use nix::libc::stat;
use crate::models::{Action, AutomationEffect, AutomationEntry, PendingAutomation};
use crate::runner::process_action::process_action;
use crate::system_state::SystemState;

/// Starts a new thread that periodically checks the system's pending automations and returns a join handle for the
/// started thread.
pub fn start_automation_processor(system_arc: Arc<Mutex<SystemState>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while !system_arc.lock().unwrap().should_exit {
            let mut system = system_arc.lock().unwrap();
            process_pending_automations(&mut system);

            thread::sleep(Duration::from_millis(10))
        }
    })
}

/// Enqueues the given automation entry for the specified service, adding all of its effects with the specified delay
/// into the service's pending automations. Removes any existing duplicate effects.
pub fn enqueue_automation(
    system: &mut SystemState,
    service_name: &str,
    automation_entry: &AutomationEntry
) {
    system.update_service_status(service_name, |status| {
        automation_entry.effects.iter().for_each(|effect| {
            // If we're queueing an effect that already exists in pending
            // automations, then we should remove it. This will create the desired
            // debounce effect.
            status.pending_automations.retain(|pending_automation| {
                pending_automation.effect != *effect
            });
            status.pending_automations.push(PendingAutomation {
                effect: *effect,
                not_before: Instant::now()
                    .add(Duration::from_millis(automation_entry.debounce_millis))
            })
        });
    })
}

/// Processes all currently pending automations for all services, firing the effects for all automations whose
/// [PendingAutomation.not_before] is less than or equal to current time.
pub fn process_pending_automations(system: &mut SystemState) {
    let check_time = Instant::now();

    system.service_statuses.iter().for_each(|(service_name, status)| {
        status.pending_automations.iter()
            .filter(|pending_automation| pending_automation.not_before.le(&check_time))
            .for_each(|pending_automation| {
                system.update_service_status(&service_name, |status| {
                    match pending_automation.effect {
                        AutomationEffect::Recompile => {
                            process_action(system, Action::Recompile(service_name.clone()));
                        }
                        AutomationEffect::Start => {
                            process_action(system, Action::UpdateRun(service_name.clone(), true));
                        },
                        AutomationEffect::Restart => {
                            process_action(system, Action::Restart(service_name.clone()));
                        }
                        AutomationEffect::Stop => {
                            process_action(system, Action::UpdateRun(service_name.clone(), false));
                        }
                        AutomationEffect::Reset => {
                            process_action(system, Action::Reset(service_name.clone()));
                        }
                    }
                });
            });

        // Remove all pending automations that were triggered
        system.update_service_status(&service_name, |status| {
            status.pending_automations.retain(|pending_automation| {
                pending_automation.not_before.gt(&check_time)
            })
        })
    });
}
