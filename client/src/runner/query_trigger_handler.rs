use crate::config::{AutomationDefinitionId, AutomationTrigger, ServiceId};
use crate::runner::scripting::engine::ScriptEngine;
use crate::system_state::SystemState;
use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use itertools::Itertools;

pub struct QueryTriggerHandler {
    state: Arc<RwLock<SystemState>>,
    script_engine: ScriptEngine,
    previous_results: RefCell<Vec<bool>>,
}
impl QueryTriggerHandler {
    pub fn new(state: Arc<RwLock<SystemState>>) -> Self {
        Self {
            state: state.clone(),
            script_engine: ScriptEngine::new(state, false),
            previous_results: RefCell::new(Vec::new()),
        }
    }

    pub fn process_automation_triggers(&mut self) {
        let mut results = self.previous_results.take();
        let mut triggered_automations: HashSet<(AutomationDefinitionId, Option<ServiceId>)> = HashSet::new();

        {
            // Process all queries in read-only mode
            let state = self.state.read().unwrap();
            // Find all automations in the current profile (either directly under the profile or under services)
            state
                .current_profile
                .iter()
                .flat_map(|profile| {
                    profile.automations.iter().chain(
                        profile
                            .services
                            .iter()
                            .flat_map(|service| service.automations.iter()),
                    )
                })
                // Find all triggers within each automation
                .flat_map(|automation| {
                    automation
                        .triggers
                        .iter()
                        .map(move |trigger| (automation, trigger))
                })
                // Remove all others except script-query based ones
                .filter(|(_, trigger)| matches!(trigger, AutomationTrigger::RhaiQuery { .. }))
                // Link each trigger to its index. Since the automations and their triggers witnin a profile are static,
                // this will always link the same trigger to the same index.
                .enumerate()
                // Process the resulting list
                .for_each(|(index, (automation, trigger))| {
                    let query = match trigger {
                        AutomationTrigger::RhaiQuery { becomes_true } => becomes_true,
                        AutomationTrigger::FileModified { .. } => panic!("FileModified trigger should have been filtered out earlier")
                    };

                    self.script_engine.set_self_service(&automation.service_id);
                    let script_result = self.script_engine.eval(query);
                    let cur_value = match script_result {
                        Ok(value) if value.as_bool().unwrap_or_default() => true,
                        _ => false
                    };

                    match results.get(index) {
                        // This is the first time the query is being executed
                        None => results.push(cur_value),
                        Some(prev_value) => {
                            // This is a moment when the query changes from false to true
                            if !prev_value && cur_value {
                                triggered_automations.insert(
                                    (automation.definition_id.clone(), automation.service_id.clone())
                                );
                            }

                            results[index] = cur_value;
                        }
                    }
                });
        }
        
        // Only lock the state for writing if we have automations to trigger
        if !triggered_automations.is_empty() {
            let mut state = self.state.write().unwrap();
            for (automation_id, service_id) in triggered_automations {
                state.update_automation(&automation_id, &service_id, |automation| {
                    automation.last_triggered = Some(Instant::now());
                });
            }
        }
    }
}
