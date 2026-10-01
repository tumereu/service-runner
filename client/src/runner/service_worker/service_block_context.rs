use crate::config::{Block, BlockId, ServiceId};
use crate::models::{BlockAction, BlockStatus, GetBlock, OutputKey, OutputKind, Service, WorkStep};
use crate::runner::scripting::executor::{RhaiRequest, ScriptExecutor};
use crate::runner::service_worker::work_context::WorkContext;
use crate::runner::service_worker::{
    ConcurrentOperationHandle, ConcurrentOperationStatus, ProcessWrapper, WorkResult, WorkWrapper,
};
use crate::system_state::{ConcurrentOperationKey, OperationType, SystemState};
use log::{debug, error, trace, warn};
use rhai::plugin::RhaiResult;
use std::ops::Deref;
use std::path::PathBuf;
use std::process::Child;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, RwLock};
use std::time::Instant;

#[derive(Clone)]
pub struct ServiceBlockContext {
    system_state: Arc<RwLock<SystemState>>,
    rhai_executor: Arc<ScriptExecutor>,
    pub service_id: ServiceId,
    pub block_id: BlockId,
}
impl ServiceBlockContext {
    pub fn new(
        system_state: Arc<RwLock<SystemState>>,
        rhai_executor: Arc<ScriptExecutor>,
        service_id: ServiceId,
        block_id: BlockId,
    ) -> Self {
        Self {
            system_state,
            rhai_executor,
            service_id,
            block_id,
        }
    }

    pub fn query_state<R, F>(&self, query: F) -> R
    where
        F: for<'a> FnOnce(&'a SystemState) -> R,
        R: 'static,
    {
        let state = self.system_state.read().unwrap();

        query(&state)
    }

    pub fn clear_current_action(&self) {
        let mut state = self.system_state.write().unwrap();
        state.update_service(&self.service_id, |service| {
            service.update_block_action(&self.block_id, None)
        })
    }

    pub fn update_status(&self, status: BlockStatus) {
        let mut state = self.system_state.write().unwrap();
        state.update_service(&self.service_id, |service| {
            let previous = service.get_block_status(&self.block_id);
            // Prerequisite retries change timestamps every half-second. Log phase transitions,
            // not each polling attempt; failures have their own throttled diagnostics.
            // discriminant compares enum variants, ignoring their fields. For two Working
            // statuses, compare the nested WorkStep variants to detect a change of phase.
            let same_phase = match (&previous, &status) {
                (BlockStatus::Working { step: old }, BlockStatus::Working { step: new }) => {
                    std::mem::discriminant(old) == std::mem::discriminant(new)
                }
                _ => std::mem::discriminant(&previous) == std::mem::discriminant(&status),
            };
            if !same_phase {
                debug!(
                    "Block {}.{}: {previous:?} -> {status:?}",
                    self.service_id, self.block_id
                );
            }
            service.update_block_status(&self.block_id, status)
        });
    }

    pub fn update_service<F>(&self, update: F)
    where
        F: for<'a> FnOnce(&'a mut Service),
    {
        self.system_state
            .write()
            .unwrap()
            .update_service(&self.service_id, update);
    }

    pub fn query_service<R, F>(&self, query: F) -> R
    where
        F: for<'a> FnOnce(&'a Service) -> R,
        R: 'static,
    {
        let state = self.system_state.read().unwrap();
        let service = state.get_service(&self.service_id).unwrap();

        query(service)
    }

    pub fn query_block<R, F>(&self, query: F) -> R
    where
        F: for<'a> FnOnce(&'a Block) -> R,
        R: 'static,
    {
        let state = self.system_state.read().unwrap();
        let block = state
            .get_service(&self.service_id)
            .unwrap()
            .get_block(&self.block_id)
            .unwrap();

        query(block)
    }

    pub fn get_action(&self) -> Option<BlockAction> {
        self.query_service(|service| service.get_block_action(&self.block_id))
    }

    pub fn get_block_status(&self) -> BlockStatus {
        self.query_service(|service| service.get_block_status(&self.block_id))
    }

    pub fn get_concurrent_operation_status(
        &self,
        operation_type: OperationType,
    ) -> Option<ConcurrentOperationStatus> {
        self.system_state
            .read()
            .unwrap()
            .get_concurrent_operation(&ConcurrentOperationKey::Block {
                service_id: self.service_id.clone(),
                block_id: self.block_id.clone(),
                operation_type,
            })
            .map(|operation| operation.status())
    }

    pub fn stop_concurrent_operation(&self, operation_type: OperationType) {
        self.system_state
            .read()
            .unwrap()
            .get_concurrent_operation(&ConcurrentOperationKey::Block {
                service_id: self.service_id.clone(),
                block_id: self.block_id.clone(),
                operation_type,
            })
            .iter()
            .for_each(|operation| operation.stop());
    }

    pub fn stop_all_operations(&self) {
        self.stop_concurrent_operation(OperationType::Check);
        self.stop_concurrent_operation(OperationType::Work);
    }

    pub fn clear_all_operations(&self) {
        [
            OperationType::Check,
            OperationType::Work,
        ].into_iter().for_each(|operation_type| {
            let debug_id = format!("{}.{}", self.service_id, self.block_id);

            match self.get_concurrent_operation_status(operation_type) {
                Some(ConcurrentOperationStatus::Running) => {
                    error!("Received request to clear stopped operation for {debug_id} but operation is still running")
                }
                Some(ConcurrentOperationStatus::Failed | ConcurrentOperationStatus::Ok) => {
                    debug!("Removing stopped operation for {debug_id}");

                    self.system_state.write().unwrap().set_concurrent_operation(
                        ConcurrentOperationKey::Block {
                            service_id: self.service_id.clone(),
                            block_id: self.block_id.clone(),
                            operation_type,
                        },
                        None,
                    );
                }
                None => {
                    // No need to do anything, no operation to remove
                }
            }
        });
    }

    pub fn create_work_context(
        &self,
        operation_type: OperationType,
        silent: bool,
    ) -> BlockWorkContext {
        BlockWorkContext {
            block_context: self,
            operation_type,
            silent,
        }
    }

    pub fn add_system_output(&self, output: String) {
        self.system_state.write().unwrap().add_output(
            &OutputKey {
                service_id: Some(self.service_id.clone()),
                source_name: self.block_id.inner().to_owned(),
                kind: OutputKind::System,
            },
            output,
        );
    }

    pub fn report_wait(&self, reason: String, now: Instant) {
        let mut state = self.system_state.write().unwrap();
        // The current waiting step owns its reminder history. Update it under the state lock
        // so concurrent check completions cannot make conflicting throttle decisions.
        let mut should_report = false;
        state.update_service(&self.service_id, |service| {
            let mut status = service.get_block_status(&self.block_id);
            if let BlockStatus::Working { step } = &mut status {
                should_report = step.record_wait_report(&reason, now);
                if should_report {
                    service.update_block_status(&self.block_id, status);
                }
            }
        });
        if !should_report {
            return;
        }
        // One throttle decision covers both the file log and the output pane.
        state.add_output(
            &OutputKey {
                service_id: Some(self.service_id.clone()),
                source_name: self.block_id.inner().to_owned(),
                kind: OutputKind::System,
            },
            format!("{reason}"),
        );
    }

    fn fingerprint_path(&self) -> PathBuf {
        let data_dir = self.query_state(|state| state.resolved_data_dir.clone());
        PathBuf::from(data_dir).join(format!(
            "{}.{}.fingerprint.md5",
            self.service_id, self.block_id
        ))
    }

    pub fn get_stored_fingerprint(&self) -> Option<String> {
        let path = self.fingerprint_path();
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                let trimmed = content.trim().to_owned();
                // Validate it looks like a hex-encoded MD5 (32 hex chars)
                if trimmed.len() == 32 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
                    Some(trimmed)
                } else {
                    warn!(
                        "Stored fingerprint at '{}' has invalid format, ignoring",
                        path.display()
                    );
                    None
                }
            }
            Err(_) => None,
        }
    }

    pub fn store_fingerprint(&self, fingerprint: &str) {
        let path = self.fingerprint_path();
        if let Err(e) = std::fs::write(&path, fingerprint) {
            error!("Failed to write fingerprint to '{}': {e}", path.display());
        }
    }

    pub fn register_external_process(&self, handle: Child, operation_type: OperationType) {
        let wrapper = ProcessWrapper::wrap(
            self.system_state.clone(),
            Some(self.service_id.clone()),
            self.block_id.inner().to_owned(),
            handle,
        );

        self.system_state.write().unwrap().set_concurrent_operation(
            ConcurrentOperationKey::Block {
                service_id: self.service_id.clone(),
                block_id: self.block_id.clone(),
                operation_type,
            },
            Some(ConcurrentOperationHandle::Process(wrapper)),
        );
    }
}

pub struct BlockWorkContext<'a> {
    block_context: &'a ServiceBlockContext,
    operation_type: OperationType,
    silent: bool,
}
impl<'a> Deref for BlockWorkContext<'a> {
    type Target = &'a ServiceBlockContext;

    fn deref(&self) -> &Self::Target {
        &self.block_context
    }
}

impl WorkContext for BlockWorkContext<'_> {
    fn stop_concurrent_operation(&self) {
        self.block_context
            .stop_concurrent_operation(self.operation_type);
    }

    fn clear_concurrent_operation(&self) {
        let debug_id = format!("{}.{}", self.service_id, self.block_id);

        match self.get_concurrent_operation_status() {
            Some(ConcurrentOperationStatus::Running) => {
                error!(
                    "Received request to clear stopped operation for {debug_id} but operation is still running"
                )
            }
            Some(ConcurrentOperationStatus::Failed | ConcurrentOperationStatus::Ok) => {
                // Checks finish on every retry; logging this cleanup at Debug would bypass
                // the throttling of waiting messages and flood the normal diagnostic log.
                trace!("Removing stopped operation for {debug_id}");

                self.system_state.write().unwrap().set_concurrent_operation(
                    ConcurrentOperationKey::Block {
                        service_id: self.service_id.clone(),
                        block_id: self.block_id.clone(),
                        operation_type: self.operation_type,
                    },
                    None,
                );
            }
            None => {
                // No need to do anything, no operation to remove
            }
        }
    }

    fn get_concurrent_operation_status(&self) -> Option<ConcurrentOperationStatus> {
        self.block_context
            .get_concurrent_operation_status(self.operation_type)
    }

    fn perform_concurrent_work<F>(&self, work: F)
    where
        F: FnOnce() -> WorkResult + Send + 'static,
    {
        // Prerequisite checks normally suppress their output because they retry frequently.
        // Capture the current requirement before launching the check so its eventual failure
        // can be reported with the correct definition and position in the requirement list.
        let prerequisite = if self.silent && matches!(self.operation_type, OperationType::Check) {
            match self.get_block_status() {
                BlockStatus::Working {
                    step:
                        WorkStep::PrerequisiteCheck {
                            checks_completed, ..
                        },
                } => self.query_block(|block| {
                    block
                        .prerequisites
                        .get(checks_completed)
                        .map(|requirement| {
                            format!(
                                "{}/{} {requirement:?}",
                                checks_completed + 1,
                                block.prerequisites.len()
                            )
                        })
                }),
                _ => None,
            }
        } else {
            None
        };
        let block_context = self.block_context.clone();
        let wrapper = WorkWrapper::wrap(
            self.system_state.clone(),
            Some(self.service_id.clone()),
            self.block_id.inner().to_owned(),
            self.silent,
            move || {
                let result = work();
                // Preserve failure details before WorkWrapper discards silent output. The
                // shared reporter handles throttling; successful polling stays quiet.
                if !result.successful {
                    if let Some(prerequisite) = prerequisite {
                        block_context.report_wait(
                            format!(
                                "Waiting for prerequisite {prerequisite}: {}",
                                result.output.join("; ")
                            ),
                            Instant::now(),
                        );
                    }
                }
                result
            },
        );
        self.system_state.write().unwrap().set_concurrent_operation(
            ConcurrentOperationKey::Block {
                service_id: self.service_id.clone(),
                block_id: self.block_id.clone(),
                operation_type: self.operation_type,
            },
            Some(ConcurrentOperationHandle::Work(wrapper)),
        );
    }

    fn register_external_process(&self, handle: Child) {
        self.block_context
            .register_external_process(handle, self.operation_type);
    }

    fn enqueue_rhai(&self, script: String, allow_fn: bool) -> Receiver<RhaiResult> {
        self.rhai_executor.enqueue(RhaiRequest {
            script,
            allow_functions: allow_fn,
            service_id: Some(self.service_id.clone()),
        })
    }

    fn add_system_output(&self, output: String) {
        self.block_context.add_system_output(output);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Settings};
    use crate::runner::service_worker::block_processor::BlockProcessor;
    use std::time::Duration;

    fn contexts() -> (ServiceBlockContext, ServiceBlockContext) {
        let service = serde_yaml::from_str(
            r#"
id: some-service
workdir: .
blocks:
  - id: build
    type: cmd-seq
    commands: []
    prerequisites:
      - type: file
        paths: [dependency]
    status_line: {symbol: C, slot: 20}
  - id: run
    type: cmd-seq
    commands: []
    status_line: {symbol: R, slot: 10}
"#,
        )
        .unwrap();
        let profile =
            serde_yaml::from_str("id: test\nworkdir: .\nservices: [{id: some-service}]").unwrap();
        let mut state = SystemState::new(
            Config {
                conf_dir: ".".into(),
                settings: Settings::default(),
                services: vec![service],
                profiles: vec![profile],
            },
            ".".into(),
        );
        state.select_profile("test");
        let state = Arc::new(RwLock::new(state));
        let executor = Arc::new(ScriptExecutor::new(state.clone()));
        let context = |block| {
            ServiceBlockContext::new(
                state.clone(),
                executor.clone(),
                ServiceId::new("some-service"),
                BlockId::new(block),
            )
        };
        (context("build"), context("run"))
    }

    fn output_count(context: &ServiceBlockContext) -> usize {
        context.query_state(|state| {
            state
                .output_store
                .outputs
                .get(&OutputKey {
                    service_id: Some(context.service_id.clone()),
                    source_name: context.block_id.inner().into(),
                    kind: OutputKind::System,
                })
                .map_or(0, |lines| lines.len())
        })
    }

    #[test]
    fn wait_reminders_are_per_step_and_reset_on_restart() {
        for step in [
            WorkStep::initial(true),
            WorkStep::ResourceGroupCheck {
                skip_work_if_healthy: true,
                last_informed_timestamp: None,
                last_informed_reason: None,
            },
        ] {
            let (build, run) = contexts();
            build.update_status(BlockStatus::Working { step: step.clone() });
            run.update_status(BlockStatus::Working { step: step.clone() });
            let now = Instant::now();
            let reason = "Waiting for a dependency";
            build.report_wait(reason.into(), now);
            assert_eq!(output_count(&build), 1);

            // Polling must not reset the clock or flood either output destination.
            for seconds in 1..300 {
                build.report_wait(reason.into(), now + Duration::from_secs(seconds));
            }
            assert_eq!(output_count(&build), 1);
            build.report_wait(reason.into(), now + Duration::from_secs(300));
            assert_eq!(output_count(&build), 2);
            build.report_wait(reason.into(), now + Duration::from_secs(301));
            assert_eq!(output_count(&build), 2);

            // New reasons and other blocks have independent initial reports.
            let changed = "Waiting for a different dependency";
            build.report_wait(changed.into(), now + Duration::from_secs(302));
            run.report_wait(changed.into(), now + Duration::from_secs(302));
            assert_eq!(output_count(&build), 3);
            assert_eq!(output_count(&run), 1);

            // Once the waiting step is gone, late diagnostics are ignored. Entering a new
            // waiting step gives a fresh reminder interval without explicit cleanup.
            build.update_status(BlockStatus::Error);
            build.report_wait(changed.into(), now + Duration::from_secs(303));
            assert_eq!(output_count(&build), 3);
            build.update_status(BlockStatus::Working { step });
            build.report_wait(changed.into(), now + Duration::from_secs(304));
            assert_eq!(output_count(&build), 4);

            // A rerun while still pending also creates a fresh prerequisite step.
            build.update_service(|service| {
                service.update_block_action(&build.block_id, Some(BlockAction::ReRun));
            });
            build.process_block();
            build.report_wait(changed.into(), now + Duration::from_secs(305));
            assert_eq!(output_count(&build), 5);
        }
    }

    #[test]
    fn prerequisite_progress_preserves_async_diagnostic_history() {
        let (build, _) = contexts();
        build.clear_current_action();
        build.update_status(BlockStatus::Working {
            step: WorkStep::initial(true),
        });
        // Complete a real asynchronous check so its diagnostic is recorded through the
        // silent-check path before the block processor consumes the result.
        build
            .create_work_context(OperationType::Check, true)
            .perform_concurrent_work(|| WorkResult {
                successful: false,
                output: vec!["dependency is missing".into()],
            });
        let join_checks = || {
            let threads = std::mem::take(&mut build.system_state.write().unwrap().active_threads);
            for (_, thread) in threads {
                thread.join().unwrap();
            }
        };
        join_checks();
        assert_eq!(output_count(&build), 1);
        let history = || match build.get_block_status() {
            BlockStatus::Working {
                step:
                    WorkStep::PrerequisiteCheck {
                        last_informed_timestamp,
                        last_informed_reason,
                        ..
                    },
            } => (
                last_informed_timestamp.unwrap(),
                last_informed_reason.unwrap(),
            ),
            other => panic!("Expected prerequisite step, got {other:?}"),
        };
        let reported = history();
        build.handle_work();
        assert_eq!(history(), reported);
        build.report_wait(reported.1.clone(), reported.0 + Duration::from_secs(1));
        assert_eq!(output_count(&build), 1);

        // Successful intermediate checks must also retain the previous reminder history.
        build.update_service(|service| {
            let mut status = service.get_block_status(&build.block_id);
            if let BlockStatus::Working {
                step: WorkStep::PrerequisiteCheck { last_failure, .. },
            } = &mut status
            {
                *last_failure = None;
                service.update_block_status(&build.block_id, status);
            }
        });
        build
            .create_work_context(OperationType::Check, true)
            .perform_concurrent_work(|| WorkResult {
                successful: true,
                output: vec![],
            });
        join_checks();
        build.handle_work();
        assert_eq!(history(), reported);
        build.handle_work();
        assert!(matches!(
            build.get_block_status(),
            BlockStatus::Working {
                step: WorkStep::ResourceGroupCheck {
                    last_informed_timestamp: None,
                    last_informed_reason: None,
                    ..
                }
            }
        ));
    }
}
