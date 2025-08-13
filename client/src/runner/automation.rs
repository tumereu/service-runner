use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use crate::config::AutomationEntry;
use crate::system_state::SystemState;

/// Starts a new thread that periodically checks the system's pending automations and returns a join handle for the
/// started thread.
pub fn start_automation_processor(system_arc: Arc<RwLock<SystemState>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while !system_arc.read().unwrap().should_exit {
            {
                let mut system = system_arc.write().unwrap();
                process_pending_automations(&mut system);
            }

            thread::sleep(Duration::from_millis(10))
        }
    })
}

/// Enqueues the given automation entry for the specified service, adding all of its effects with the specified delay
/// into the service's pending automations. Removes any existing duplicate effects. Respects the
/// [ServiceStatus::automation_enabled] and [ServiceStatus::automation_modes] for the specific entry.
pub fn enqueue_automation(
    system: &mut SystemState,
    service_name: &str,
    automation_entry: &AutomationEntry
) {
    let automation_name = &automation_entry.name;
    /*
    FIXME
    let current_mode: AutomationMode = system.get_service_status(service_name)
        .and_then(|status| status.automation_modes.get(&automation_entry.name)).copied()
        .unwrap_or(AutomationMode::Disabled);

    if !system.get_service_status(service_name).map(|status| status.automation_enabled).unwrap_or(false) {
        debug!("Ignoring triggered automation {automation_name} for {service_name} as automation is disabled for the service");
    } else if current_mode == AutomationMode::Disabled {
        debug!("Ignoring triggered automation {automation_name} for {service_name} that automation specifically is disabled");
    } else {
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
                    // Set the debounce time to a thousand years if the automation is currently triggerable
                    not_before: if current_mode == AutomationMode::Triggerable {
                        Instant::now().add(Duration::from_secs(3600 * 24 * 365 * 1000))
                    } else {
                        Instant::now()
                            .add(Duration::from_millis(automation_entry.debounce_millis))
                    }
                })
            });
        });
    }
    
     */
}

/// Processes all currently pending automations for all services, firing the effects for all automations whose
/// [PendingAutomation.not_before] is less than or equal to current time.
pub fn process_pending_automations(system: &mut SystemState) {
    let check_time = Instant::now();

    /*
    FIXME
    let triggered_automations: Vec<(String, PendingAutomation)> = system.service_statuses.iter()
        .flat_map(|(service_name, status)| {
            status.pending_automations.iter()
                .filter(|pending_automation| pending_automation.not_before.le(&check_time))
                .map(|pending_automation| {
                    (service_name.clone(), pending_automation.clone())
                })
        }).collect();
    triggered_automations.iter().for_each(|(service_name, pending_automation)| {
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

    // Remove all pending automations that were triggered
    system.service_statuses.iter_mut().for_each(|(_service_name, status)| {
        status.pending_automations.retain(|pending_automation| {
            pending_automation.not_before.gt(&check_time)
        })
    });
    
     */
}
