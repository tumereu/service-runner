use crate::config::{ExecutableEntry, Requirement, TaskStep};
use crate::runner::service_worker::requirement_checker::{RequirementCheckResult, RequirementChecker};
use crate::runner::service_worker::work_context::WorkContext;
use crate::runner::service_worker::{ConcurrentOperationStatus, WorkResult};
use crate::utils::format_err;
use std::time::{Duration, Instant};
use crate::runner::service_worker::create_cmd::create_cmd;

pub enum WorkExecutionResult {
    EntryOk,
    AllOk,
    Working,
    RecoverableFailure,
    Failed,
}

pub struct WorkSequenceExecutor<'a, W: WorkContext> {
    pub sequence: Vec<WorkSequenceEntry>,
    pub completed_count: usize,
    pub entry_start_time: Instant,
    pub last_recoverable_failure: Option<Instant>,
    pub context: &'a W,
    pub workdir: String,
}
impl<'a, W: WorkContext> WorkSequenceExecutor<'a, W> {
    pub fn exec_next(self) -> WorkExecutionResult {
        let next_entry = &self.sequence.get(self.completed_count);

        match next_entry {
            None => WorkExecutionResult::AllOk,
            Some(WorkSequenceEntry::ExecutableEntry(entry)) => self.handle_executable_entry(entry),
            Some(WorkSequenceEntry::RhaiScript(script)) => self.handle_rhai_script(script.clone()),
            Some(WorkSequenceEntry::WaitRequirement { timeout, requirement }) => self.handle_requirement(timeout, requirement),
        }
    }

    fn handle_executable_entry(&self, entry: &ExecutableEntry) -> WorkExecutionResult {
        match self.context.get_concurrent_operation_status() {
            None => {
                match create_cmd(entry, Some(self.workdir.clone())) {
                    Ok(mut command) => {
                        self.context.add_system_output(format!("Exec: {entry}"));

                        match command.spawn() {
                            Ok(process_handle) => {
                                self.context.register_external_process(process_handle);
                                WorkExecutionResult::Working
                            }
                            Err(error) => {
                                self.context.add_system_output(format_err!("Failed to spawn child process", error));
                                WorkExecutionResult::Failed
                            }
                        }
                    },
                    Err(error) => {
                        self.context.add_system_output(format_err!("Error in command creation", error));
                        WorkExecutionResult::Failed
                    }
                }
            }
            Some(ConcurrentOperationStatus::Running) => WorkExecutionResult::Working,
            Some(ConcurrentOperationStatus::Ok) => {
                self.context.clear_concurrent_operation();
                WorkExecutionResult::EntryOk
            }
            Some(ConcurrentOperationStatus::Failed) => {
                self.context.clear_concurrent_operation();
                WorkExecutionResult::Failed
            }
        }
    }

    fn handle_rhai_script(&self, script: String) -> WorkExecutionResult {
        match self.context.get_concurrent_operation_status() {
            None => {
                let result_rx = self.context.enqueue_rhai(script.clone(), true);

                self.context.perform_concurrent_work(move || {
                    match result_rx.recv() {
                        Ok(Ok(_)) => WorkResult {
                            successful: true,
                            output: vec![format!("Rhai script OK: {}", script)]
                        },
                        Ok(Err(error)) => WorkResult {
                            successful: false,
                            output: vec![format!("Error in Rhai script {script}: {error:?}")],
                        },
                        Err(error) => WorkResult {
                            successful: false,
                            output: vec![format!("Error in receiving response from Rhai executor: {error:?}")],
                        },
                    }
                });
                WorkExecutionResult::Working
            }
            Some(ConcurrentOperationStatus::Running) => WorkExecutionResult::Working,
            Some(ConcurrentOperationStatus::Ok) => {
                self.context.clear_concurrent_operation();
                WorkExecutionResult::EntryOk
            }
            Some(ConcurrentOperationStatus::Failed) => {
                self.context.clear_concurrent_operation();
                WorkExecutionResult::Failed
            }
        }
    }

    fn handle_requirement(&self, timeout: &Duration, requirement: &Requirement) -> WorkExecutionResult {
        let result = RequirementChecker {
            all_requirements: vec![requirement.clone()],
            completed_count: 0,
            timeout: Some(timeout.clone()),
            failure_wait_time: WAIT_REQUIREMENT_FAILURE_WAIT,
            start_time: self.entry_start_time,
            last_failure: self.last_recoverable_failure,
            context: self.context,
            workdir: self.workdir.clone(),
        }.check_requirements();

        match result {
            RequirementCheckResult::AllOk | RequirementCheckResult::CurrentCheckOk => WorkExecutionResult::EntryOk,
            RequirementCheckResult::CurrentCheckFailed => WorkExecutionResult::RecoverableFailure,
            RequirementCheckResult::Timeout => WorkExecutionResult::Failed,
            RequirementCheckResult::Working => WorkExecutionResult::Working,
        }
    }
}

pub enum WorkSequenceEntry {
    ExecutableEntry(ExecutableEntry),
    RhaiScript(String),
    WaitRequirement {
        timeout: Duration,
        requirement: Requirement,
    },
}
impl Into<WorkSequenceEntry> for ExecutableEntry {
    fn into(self) -> WorkSequenceEntry {
        WorkSequenceEntry::ExecutableEntry(self)
    }
}
impl Into<WorkSequenceEntry> for TaskStep {
    fn into(self) -> WorkSequenceEntry {
        match self {
            TaskStep::Command { command } => WorkSequenceEntry::ExecutableEntry(command),
            TaskStep::Action { action } => WorkSequenceEntry::RhaiScript(action),
            TaskStep::Wait { timeout, requirement } => WorkSequenceEntry::WaitRequirement {
                timeout,
                requirement,
            },
        }
    }
}

const WAIT_REQUIREMENT_FAILURE_WAIT: Duration = Duration::from_millis(3000);
