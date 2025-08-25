use crate::config::{AutomationDefinitionId, AutomationTrigger, ServiceId};
use crate::runner::scripting::engine::ScriptEngine;
use crate::system_state::SystemState;
use itertools::Itertools;
use log::debug;
use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use std::time::Instant;

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
        struct ProcessableQuery {
            automation_id: AutomationDefinitionId,
            service_id: Option<ServiceId>,
            index: usize,
            query: String,
        }

        let queries_to_process: Vec<ProcessableQuery> = {
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
                // Collect each query into a struct so we can release the state-lock
                .map(|(index, (automation, trigger))| {
                    let query = match trigger {
                        AutomationTrigger::RhaiQuery { becomes_true } => becomes_true,
                        AutomationTrigger::FileModified { .. } => {
                            panic!("FileModified trigger should have been filtered out earlier")
                        }
                    };

                    ProcessableQuery {
                        automation_id: automation.definition_id.clone(),
                        service_id: automation.service_id.clone(),
                        index,
                        query: query.clone(),
                    }
                })
                .collect()
        };

        let mut results = self.previous_results.take();
        let mut to_trigger: HashSet<(AutomationDefinitionId, Option<ServiceId>)> = HashSet::new();

        // Then process the resulting list
        for processable_query in queries_to_process {
            let ProcessableQuery {
                automation_id,
                service_id,
                index,
                query,
            } = processable_query;

            self.script_engine.set_self_service(&service_id);
            let script_result = self.script_engine.eval(&query);
            let cur_value = match script_result {
                Ok(value) if value.as_bool().unwrap_or_default() => true,
                _ => false,
            };

            match results.get(index) {
                // This is the first time the query is being executed
                None => results.push(cur_value),
                Some(prev_value) => {
                    // This is a moment when the query changes from false to true
                    if !prev_value && cur_value {
                        to_trigger.insert((automation_id, service_id));
                    }

                    results[index] = cur_value;
                }
            }
        }

        // Only lock the state for writing if we have automations to trigger
        if !to_trigger.is_empty() {
            let mut state = self.state.write().unwrap();
            for (automation_id, service_id) in to_trigger {
                state.update_automation(&automation_id, &service_id, |automation| {
                    automation.last_triggered = Some(Instant::now());
                });
            }
        }
    }
}
