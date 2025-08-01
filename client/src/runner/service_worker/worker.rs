use std::net::TcpListener;
use crate::config::{Block, HttpMethod, RequiredStatus, Requirement, WorkDefinition};
use crate::models::{BlockAction, BlockStatus, GetBlock, OutputKey, OutputKind, Service, WorkStep};
use crate::runner::service_worker::process_wrapper::{create_cmd, ProcessWrapper};
use crate::runner::service_worker::{
    AsyncOperationHandle, AsyncOperationStatus, CtrlOutputWriter, WorkWrapper,
};
use crate::system_state::SystemState;
use crate::utils::format_err;
use log::{debug, error, info};
use reqwest::blocking::Client as HttpClient;
use reqwest::Method;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use crate::runner::service_worker::block_worker::BlockWorker;

pub fn work_services(state_arc: Arc<Mutex<SystemState>>) {
    // A collection of (service_id, block_id) pairs describing all services and their blocks
    // that might need to be worked on.
    let stages_to_work = {
        let state = state_arc.lock().unwrap();

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

    // Loop through all information we collected previously and launch appropriate subprocesses to
    // work them.
    stages_to_work
        .into_iter()
        .for_each(|(service_id, block_id)| {
            work_block(state_arc.clone(), &service_id, &block_id);
        });
}

trait WorkBlock {
    fn work_block(&self);
}

impl WorkBlock for BlockWorker {
    fn work_block(&self) {
        let service_enabled = self.state_arc
            .lock()
            .unwrap()
            .get_service(&self.service_id)
            .unwrap()
            .enabled;

        let (status, action) = {
            let state = state_arc.lock().unwrap();
            let service = state.get_service(&service_id).unwrap();

            (
                service.get_block_status(&block_id).clone(),
                service.get_block_action(&block_id).clone(),
            )
        };

        // TODO check dependencies?
        match (status, action) {
            (_, Some(BlockAction::Enable)) => {
                clear_current_action(state_arc.clone(), service_id, block_id);
                state_arc
                    .lock()
                    .unwrap()
                    .update_service(service_id, |service| {
                        service.enabled = true;
                    })
            }
            (_, Some(BlockAction::ToggleEnabled)) if !service_enabled => {
                clear_current_action(state_arc.clone(), service_id, block_id);
                state_arc
                    .lock()
                    .unwrap()
                    .update_service(service_id, |service| {
                        service.enabled = true;
                    })
            }

            (_, Some(BlockAction::Disable)) => {
                stop_block_operation_and_then(state_arc.clone(), service_id, block_id, || {
                    clear_current_action(state_arc.clone(), service_id, block_id);
                    state_arc
                        .lock()
                        .unwrap()
                        .update_service(service_id, |service| {
                            service.enabled = false;
                        });
                });
            }
            (_, Some(BlockAction::ToggleEnabled)) => {
                stop_block_operation_and_then(state_arc.clone(), service_id, block_id, || {
                    clear_current_action(state_arc.clone(), service_id, block_id);
                    state_arc
                        .lock()
                        .unwrap()
                        .update_service(service_id, |service| {
                            service.enabled = false;
                        });
                });
            }

            (_, Some(_)) if !service_enabled => {
                clear_current_action(state_arc.clone(), service_id, block_id);
            }

            (BlockStatus::Working { .. }, None) => {
                handle_work(state_arc.clone(), service_id, block_id);
            }

            (_, Some(BlockAction::ReRun)) => {
                stop_block_operation_and_then(state_arc.clone(), service_id, block_id, || {
                    clear_current_action(state_arc.clone(), service_id, block_id);
                    update_status(
                        state_arc.clone(),
                        service_id,
                        block_id,
                        BlockStatus::Working {
                            skip_if_healthy: false,
                            step: WorkStep::default(),
                        },
                    )
                });
            }

            (BlockStatus::Initial | BlockStatus::Error, Some(BlockAction::Run)) => {
                info!("Block {service_id}.{block_id} will be run");
                clear_current_action(state_arc.clone(), service_id, block_id);

                update_status(
                    state_arc.clone(),
                    service_id,
                    block_id,
                    BlockStatus::Working {
                        skip_if_healthy: true,
                        step: WorkStep::default(),
                    },
                )
            }

            (BlockStatus::Working { .. } | BlockStatus::Ok, Some(BlockAction::Run)) => {
                info!("Block {service_id}.{block_id} is already in a running/finished status, clearing run-action");
                clear_current_action(state_arc.clone(), service_id, block_id);
            }
            (status, Some(BlockAction::Stop)) => {
                stop_block_operation_and_then(state_arc.clone(), service_id, block_id, || {
                    clear_current_action(state_arc.clone(), service_id, block_id);
                    update_status(
                        state_arc.clone(),
                        service_id,
                        block_id,
                        match status {
                            BlockStatus::Initial => BlockStatus::Initial,
                            BlockStatus::Working { .. } => BlockStatus::Initial,
                            // FIXME maybe this should be based on work type, or maybe health check?
                            //       in any case, we can't always go back to initial. Or can we?
                            BlockStatus::Ok => BlockStatus::Initial,
                            BlockStatus::Error => BlockStatus::Error,
                        },
                    )
                });
            }
            (BlockStatus::Working { .. }, Some(BlockAction::Cancel)) => {
                stop_block_operation_and_then(state_arc.clone(), service_id, block_id, || {
                    clear_current_action(state_arc.clone(), service_id, block_id);
                });
            }
            (
                BlockStatus::Initial | BlockStatus::Ok | BlockStatus::Error,
                Some(BlockAction::Cancel),
            ) => {
                clear_current_action(state_arc.clone(), service_id, block_id);
            }

            (_, None) => {
                // Intentionally do nothing: we're either currently performing some work, or are in some
                // other state with no action to execute
            }
        }
    }
}

fn exec_next_work(
    state_arc: Arc<Mutex<SystemState>>,
    service_id: &str,
    block_id: &str,
    steps_completed: usize,
) {
    let work = {
        let state = state_arc.lock().unwrap();

        state
            .get_service(&service_id)
            .unwrap()
            .get_block(&block_id)
            .unwrap()
            .work
            .clone()
    };

    match work {
        WorkDefinition::CommandSeq { commands } => {
            let next_command = &commands[steps_completed];
            let mut command = create_cmd(
                next_command,
                Some(
                    state_arc
                        .lock()
                        .unwrap()
                        .get_service(&service_id)
                        .unwrap()
                        .definition
                        .dir
                        .clone(),
                ),
            );

            {
                let mut state = state_arc.lock().unwrap();
                state.add_output(
                    &OutputKey {
                        name: OutputKey::CTL.into(),
                        service_ref: service_id.to_owned(),
                        kind: OutputKind::Compile,
                    },
                    format!("Exec: {next_command}"),
                );
            }

            match command.spawn() {
                Ok(process_handle) => {
                    let wrapper = ProcessWrapper::wrap(
                        state_arc.clone(),
                        process_handle,
                        service_id.to_owned(),
                        block_id.to_owned(),
                    );

                    let mut state = state_arc.lock().unwrap();
                    state.set_block_operation(
                        service_id,
                        block_id,
                        Some(AsyncOperationHandle::Process(wrapper)),
                    );
                }
                Err(error) => {
                    let mut state = state_arc.lock().unwrap();
                    state.update_service(&service_id, |service| {
                        service.update_block_status(&block_id, BlockStatus::Error)
                    });

                    state.add_output(
                        &OutputKey {
                            name: OutputKey::CTL.into(),
                            service_ref: service_id.to_owned(),
                            kind: OutputKind::Compile,
                        },
                        format_err!("Failed to spawn child process", error),
                    );
                }
            }
        }
        WorkDefinition::Process { executable } => {
            // TODO handle
        }
    }
}

fn stop_block_operation_and_then<F>(
    state_arc: Arc<Mutex<SystemState>>,
    service_id: &str,
    block_id: &str,
    execute: F,
) where
    F: FnOnce(),
{
    let process_status = state_arc
        .lock()
        .unwrap()
        .get_block_operation(service_id, block_id)
        .map(|operation| operation.status());

    match process_status {
        Some(AsyncOperationStatus::Running) => {
            debug!("Stopping current operation for {service_id}.{block_id}");
            state_arc
                .lock()
                .unwrap()
                .get_block_operation(service_id, block_id)
                .iter()
                .for_each(|operation| operation.stop());
        }
        Some(status) => {
            debug!("Current operation for {service_id}.{block_id} has stopped ({status:?}), removing it");

            state_arc
                .lock()
                .unwrap()
                .set_block_operation(service_id, block_id, None)
        }
        None => {
            execute();
        }
    }
}

fn handle_work(state_arc: Arc<Mutex<SystemState>>, service_id: &str, block_id: &str) {
    let status = state_arc
        .lock()
        .unwrap()
        .get_service(&service_id)
        .unwrap()
        .get_block_status(&block_id)
        .clone();

    let work = state_arc
        .lock()
        .unwrap()
        .get_service(&service_id)
        .unwrap()
        .get_block(&block_id)
        .unwrap()
        .work
        .clone();

    let (step, skip_if_healthy) = match status {
        BlockStatus::Working {
            step,
            skip_if_healthy,
        } => (step, skip_if_healthy),
        _ => {
            error!("ERROR: invoked work-processing function with invalid block status {status:?}");
            return;
        }
    };

    let current_process_status = state_arc
        .lock()
        .unwrap()
        .get_block_operation(service_id, block_id)
        .map(|operation| operation.status());

    match step {
        // Ensure that there's no lingering process. There should not be if other actions are handled correctly,
        // but some defensive programming here doesn't hurt.
        WorkStep::Initial => {
            stop_block_operation_and_then(state_arc.clone(), service_id, block_id, || {
                update_status(
                    state_arc.clone(),
                    service_id,
                    block_id,
                    BlockStatus::Working {
                        skip_if_healthy,
                        step: WorkStep::PrerequisiteCheck {
                            checks_completed: 0,
                            last_failure: None,
                        },
                    },
                );
            })
        }

        WorkStep::PrerequisiteCheck {
            checks_completed,
            last_failure,
        } => {
            let current_requirement = state_arc
                .lock()
                .unwrap()
                .get_service_block(service_id, block_id)
                .and_then(|block| block.prerequisites.get(checks_completed))
                .map(|requirement| requirement.clone());

            // TODO account for last failure time

            match (current_process_status, current_requirement) {
                (_, None) if skip_if_healthy => {
                    update_status(
                        state_arc.clone(),
                        service_id,
                        block_id,
                        BlockStatus::Working {
                            skip_if_healthy,
                            step: WorkStep::PreWorkHealthCheck {
                                checks_completed: 0,
                            },
                        },
                    );
                }
                (_, None) => {
                    update_status(
                        state_arc.clone(),
                        service_id,
                        block_id,
                        BlockStatus::Working {
                            skip_if_healthy,
                            step: WorkStep::PerformWork {
                                steps_completed: 0
                            },
                        },
                    );
                }
                (None, Some(requirement)) => {
                    WorkWrapper::wrap(
                        state_arc.clone(),
                        service_id.to_owned(),
                        block_id.to_owned(),
                        || check_requirement(state_arc.clone(), service_id, block_id, &requirement),
                    );
                }
                (Some(AsyncOperationStatus::Failed), _) => {
                    update_status(
                        state_arc.clone(),
                        service_id,
                        block_id,
                        BlockStatus::Working {
                            skip_if_healthy,
                            step: WorkStep::PrerequisiteCheck {
                                checks_completed: 0,
                                last_failure: Some(Instant::now()),
                            },
                        },
                    );
                }
            }
        }
    }

    pub enum RequirementCheckResult {
        Ok,
        Failed,
        Async,
    }

    fn check_requirement(
        state_arc: Arc<Mutex<SystemState>>,
        service_id: &str,
        block_id: &str,
        requirement: &Requirement,
    ) -> bool {
        match requirement {
            Requirement::Http {
                url,
                method,
                timeout_millis,
                status,
            } => {
                let state_arc = state_arc.clone();
                let http_client = HttpClient::new();

                let result = http_client
                    .request(
                        match method {
                            HttpMethod::GET => Method::GET,
                            HttpMethod::POST => Method::POST,
                            HttpMethod::PUT => Method::PUT,
                            HttpMethod::PATCH => Method::PATCH,
                            HttpMethod::DELETE => Method::DELETE,
                            HttpMethod::OPTIONS => Method::OPTIONS,
                        },
                        url,
                    )
                    .timeout(Duration::from_millis(*timeout_millis))
                    .send();

                // TODO include http call in logs, also block and service id?
                let mut state = state_arc.lock().unwrap();
                if let Ok(response) = result {
                    let response_status: u16 = response.status().into();
                    if response_status != *status {
                        state.add_ctrl_output(
                            service_id,
                            format!(
                                "Requirement failed: HTTP status {actual} != {expected}",
                                actual = response_status,
                                expected = status
                            ),
                        );

                        false
                    } else {
                        state.add_ctrl_output(
                            service_id,
                            format!(
                                "Requirement OK: HTTP status {actual} == {expected}",
                                actual = response_status,
                                expected = status
                            ),
                        );

                        true
                    }
                } else {
                    state.add_ctrl_output(
                        service_id,
                        "Requirement failed: HTTP request timeout".to_string(),
                    );

                    false
                }
            }
            Requirement::Port { port, host } => {
                let host = match host {
                    Some(host) => host.clone(),
                    None => "127.0.0.1".to_owned()
                };

                if TcpListener::bind(format!("{host}:{port}")).is_err() {
                    let mut state = state_arc.lock().unwrap();

                    state.add_ctrl_output(
                        service_id,
                        format!("Requirement failed: port {host}:{port} not open"),
                    );

                    false
                } else {
                    let mut state = state_arc.lock().unwrap();

                    state.add_ctrl_output(
                        service_id,
                        format!("Requirement OK: port {host}:{port} is open"),
                    );

                    true
                }
            },
            Requirement::Dependency { service: required_service, block: block_ref, status: required_status } => {
                let state = state_arc.lock().unwrap();

                let result = state
                    .iter_services()
                    // Find the service the prerequisite refers to
                    .find(|service| match required_service {
                        // Default to the service itself
                        None => service.definition.id == service_id,
                        Some(req_service_id) => &service.definition.id == req_service_id
                    })
                    .map(|service| {
                        // Check that the status is acceptable according to the required status of the prereq
                        match service.get_block_status(block_ref) {
                            BlockStatus::Initial => *required_status == RequiredStatus::Initial,
                            BlockStatus::Working { .. } => *required_status == RequiredStatus::Working,
                            BlockStatus::Ok => *required_status == RequiredStatus::Ok,
                            BlockStatus::Error => *required_status == RequiredStatus::Error,
                        }
                    })
                    .unwrap_or(false);

                result
            }
        }
    }
}

const PRE_REQ_FAILURE_WAIT: Duration = Duration::from_secs(5);