use crate::models::{Automation, Task, TaskStatus};
use crate::runner::scripting::executor::ScriptExecutor;
use crate::runner::service_worker::block_processor::BlockProcessor;
use crate::runner::service_worker::service_block_context::ServiceBlockContext;
use crate::runner::service_worker::task_context::TaskContext;
use crate::system_state::SystemState;
pub use concurrent_operation::*;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use log::{debug, info};
use crate::config::{AutomationAction, AutomationDefinitionId, ServiceId};
use crate::runner::query_trigger_handler::QueryTriggerHandler;
use crate::runner::service_worker::task_processor::TaskProcessor;
use crate::runner::service_worker::ConcurrentOperationStatus;

mod concurrent_operation;
mod service_block_context;
mod block_processor;
mod requirement_checker;
mod work_context;
mod work_sequence_executor;
mod task_context;
mod task_processor;

pub struct ServiceWorker {
    state: Arc<RwLock<SystemState>>,
    rhai_executor: Arc<ScriptExecutor>,
    keep_running: Arc<Mutex<bool>>,
}
impl ServiceWorker {
    pub fn new(state: Arc<RwLock<SystemState>>, rhai_executor: Arc<ScriptExecutor>) -> Self {
        Self {
            state: state.clone(),
            rhai_executor: rhai_executor.clone(),
            keep_running: Arc::new(Mutex::new(true)),
        }
    }

    pub fn start(&self) -> JoinHandle<()> {
        let keep_running = self.keep_running.clone();
        let state = self.state.clone();
        let executor = self.rhai_executor.clone();

        thread::spawn(move || {
            let mut query_trigger_handler = QueryTriggerHandler::new(state.clone());

            while *keep_running.lock().unwrap() {
                Self::work_services(state.clone(), executor.clone());
                query_trigger_handler.process_automation_triggers();
                Self::spawn_automation_tasks(state.clone());

                thread::sleep(Duration::from_millis(30))
            }
        })
    }

    pub fn stop(&self) {
        *self.keep_running.lock().unwrap() = false;
    }

    fn work_services(
        state_arc: Arc<RwLock<SystemState>>,
        rhai_executor: Arc<ScriptExecutor>,
    ) {
        // A collection of (service_id, block_id) pairs describing all services and their blocks
        // that might need to be worked on.
        let blocks_to_work = {
            let state = state_arc.read().unwrap();

            state
                .iter_services()
                .flat_map(|service| {
                    service
                        .definition
                        .blocks
                        .iter()
                        .map(|block| (service.definition.id.clone(), block.id.clone()))
                })
                .collect::<Vec<_>>()
        };

        let tasks_to_work = {
            let state = state_arc.read().unwrap();

            state.current_profile.iter().flat_map(|profile| profile.running_tasks.iter())
                .filter(|task| matches!(task.status, TaskStatus::Running { .. }))
                .map(|task| task.id)
                .collect::<Vec<_>>()
        };

        // Loop through all information we collected previously and launch appropriate subprocesses to
        // work them.
        blocks_to_work
            .into_iter()
            .for_each(|(service_id, block_id)| {
                ServiceBlockContext::new(
                    state_arc.clone(),
                    rhai_executor.clone(),
                    service_id,
                    block_id,
                ).process_block();
            });

        tasks_to_work.into_iter().for_each(|task_id| {
            TaskContext::new(
                state_arc.clone(),
                rhai_executor.clone(),
                task_id,
            ).process_task();
        });

        // Clean up, remove finished tasks
        state_arc.write().unwrap().current_profile
            .iter_mut().for_each(|profile| {
            profile.running_tasks.retain(|task| {
                matches!(task.status, TaskStatus::Running { .. })
            });
        });
    }

    fn spawn_automation_tasks(
        state_arc: Arc<RwLock<SystemState>>,
    ) {
        struct TriggerableAutomation {
            automation_id: AutomationDefinitionId,
            service_id: Option<ServiceId>,
            action: AutomationAction,
        }

        let to_trigger: Vec<TriggerableAutomation> = {
            // Process finding what to trigger in read-only mode
            let state = state_arc.read().unwrap();
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
                // Find all automations that should be processed
                .filter(|automation| match automation.last_triggered {
                    Some(triggered_on) => Instant::now().duration_since(triggered_on) >= automation.debounce,
                    None => false,
                })
                // Collect ids into a struct so we can release the state-lock
                .map(|(automation)| {
                    TriggerableAutomation {
                        automation_id: automation.definition_id.clone(),
                        service_id: automation.service_id.clone(),
                        action: automation.action.clone(),
                    }
                })
                .collect()
        };

        if !to_trigger.is_empty() {
            // Only write-lock the state if we have changes to make
            let mut state = state_arc.write().unwrap();

            for triggerable in to_trigger {
                // Perform the action, spawning the necessary tasks
                match triggerable.action {
                    AutomationAction::RunOwnTask { id } => {
                        info!(
                            "Spawning own task {id} for {service_id:?} as a result of automation {automation:?}",
                            id = id,
                            service_id = triggerable.service_id,
                            automation = triggerable.automation_id,
                        );
                        state.update_profile(|profile| {
                            profile.spawn_task(&id, triggerable.service_id.clone());
                        })
                    },
                    AutomationAction::RunAnyTask { id, service } => {
                        info!(
                            "Spawning task {id} for {service_id:?} as a result of automation {automation:?}",
                            id = id,
                            service_id = service,
                            automation = triggerable.automation_id,
                        );
                        state.update_profile(|profile| {
                            profile.spawn_task(&id, service.clone());
                        })
                    }
                    AutomationAction::InlineTask { steps } => {
                        info!(
                            "Spawning inline task a result of automation {automation:?}",
                            automation = triggerable.automation_id,
                        );
                        state.update_profile(|profile| {
                            profile.spawn_inline_task(
                                triggerable.service_id.clone(),
                                steps,
                                triggerable.automation_id.0.clone(),
                            );
                        })
                    }
                }

                // Reset last_triggered timestamp
                match triggerable.service_id.as_ref() {
                    Some(service_id) => state.update_service(service_id, |service| {
                        service.update_automation(&triggerable.automation_id, |automation| {
                            automation.last_triggered = None;
                        })
                    }),
                    None => {
                        state.update_profile(|profile| {
                            profile.update_automation(&triggerable.automation_id, |automation| {
                                automation.last_triggered = None;
                            })
                        })
                    }
                }
            }
        }
    }
}

