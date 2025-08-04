use reqwest::blocking::Client as HttpClient;
use reqwest::Method;
use std::net::{TcpListener};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use crate::config::{ExecutableEntry, HttpMethod, Requirement, TaskStep};
use crate::rhai::RHAI_ENGINE;
use crate::runner::service_worker::{ConcurrentOperationStatus, WorkResult};
use crate::runner::service_worker::requirement_checker::{RequirementCheckResult, RequirementChecker};
use crate::runner::service_worker::work_context::WorkContext;
use crate::system_state::OperationType;
use crate::utils::format_err;

pub enum SequenceExecutionResult {
    EntryOk,
    AllOk,
    Working,
    RecoverableFailure,
    Failed,
}

pub struct SequenceExecutor<'a, W: WorkContext> {
    pub sequence: Vec<SequenceEntry>,
    pub completed_count: usize,
    pub start_time: Instant,
    pub last_recoverable_failure: Option<Instant>,
    pub context: &'a W,
    pub workdir: String,
}
impl<'a, W: WorkContext> SequenceExecutor<'a, W> {
    pub fn exec_next(self) -> SequenceExecutionResult {
        let next_entry = &self.sequence.get(self.completed_count);

        match next_entry {
            None => SequenceExecutionResult::AllOk,
            Some(SequenceEntry::ExecutableEntry(entry)) => self.handle_executable_entry(entry),
            Some(SequenceEntry::RhaiScript(script)) => self.handle_rhai_script(script),
            Some(SequenceEntry::WaitRequirement { timeout, requirement }) => self.handle_requirement(timeout, requirement),
        }
    }

    fn handle_executable_entry(&self, entry: &ExecutableEntry) -> SequenceExecutionResult {
        match self.context.get_concurrent_operation_status(OperationType::Work) {
            None => {
                let mut command = create_cmd(entry, Some(self.workdir.clone()));
                self.context.add_ctrl_output(format!("Exec: {entry}"));

                match command.spawn() {
                    Ok(process_handle) => {
                        self.context.register_external_process(process_handle, OperationType::Work);
                        SequenceExecutionResult::Working
                    }
                    Err(error) => {
                        self.context.add_ctrl_output(format_err!("Failed to spawn child process", error));
                        SequenceExecutionResult::Failed
                    }
                }
            }
            Some(ConcurrentOperationStatus::Running) => SequenceExecutionResult::Working,
            Some(ConcurrentOperationStatus::Ok) => {
                self.context.clear_concurrent_operation(OperationType::Work);
                SequenceExecutionResult::EntryOk
            }
            Some(ConcurrentOperationStatus::Failed) => {
                self.context.clear_concurrent_operation(OperationType::Work);
                SequenceExecutionResult::Failed
            }
        }
    }

    fn handle_rhai_script(&self, script: &String) -> SequenceExecutionResult {
        let mut scope = self.context.create_rhai_scope();
        // TODO currently evaluation is performed synchronously. Move engine to a worker thread to allow for
        //      longer scripts?
        match RHAI_ENGINE.eval_with_scope::<bool>(&mut scope, script) {
            Ok(_) => SequenceExecutionResult::EntryOk,
            Err(_) => SequenceExecutionResult::Failed,
        }
    }

    fn handle_requirement(&self, timeout: &Duration, requirement: &Requirement) -> SequenceExecutionResult {
        let result = RequirementChecker {
            all_requirements: vec![requirement.clone()],
            completed_count: 0,
            timeout: Some(timeout.clone()),
            failure_wait_time: WAIT_REQUIREMENT_FAILURE_WAIT,
            start_time: self.start_time,
            last_failure: self.last_recoverable_failure,
            context: self.context,
            workdir: self.workdir.clone(),
        }.check_requirements();

        match result {
            RequirementCheckResult::AllOk | RequirementCheckResult::CurrentCheckOk => SequenceExecutionResult::EntryOk,
            RequirementCheckResult::CurrentCheckFailed => SequenceExecutionResult::RecoverableFailure,
            RequirementCheckResult::Timeout => SequenceExecutionResult::Failed,
            RequirementCheckResult::Working => SequenceExecutionResult::Working,
        }
    }
}

pub enum SequenceEntry {
    ExecutableEntry(ExecutableEntry),
    RhaiScript(String),
    WaitRequirement {
        timeout: Duration,
        requirement: Requirement,
    },
}
impl Into<SequenceEntry> for ExecutableEntry {
    fn into(self) -> SequenceEntry {
        SequenceEntry::ExecutableEntry(self)
    }
}
impl Into<SequenceEntry> for TaskStep {
    fn into(self) -> SequenceEntry {
        match self {
            TaskStep::Command { command } => SequenceEntry::ExecutableEntry(command),
            TaskStep::Action { action } => SequenceEntry::RhaiScript(action),
            TaskStep::Wait { timeout, requirement } => SequenceEntry::WaitRequirement {
                timeout,
                requirement,
            },
        }
    }
}

const WAIT_REQUIREMENT_FAILURE_WAIT: Duration = Duration::from_millis(3000);

pub fn create_cmd<S>(entry: &ExecutableEntry, dir: Option<S>) -> Command
where
    S: AsRef<str>,
{
    let mut cmd = Command::new(entry.executable.clone());
    cmd.args(entry.args.clone());
    if let Some(dir) = dir {
        cmd.current_dir(dir.as_ref());
    }
    entry.env.iter().for_each(|(key, value)| {
        // Substitute environment variables if placeholders are used in the env entry
        // TODO clean error handling, bubble error up and process in a nice way above
        let parsed = subst::substitute(value, &subst::Env)
            .expect(&format!("No variable found to substitute in env variable {}", value));

        cmd.env(key.clone(), parsed);
    });
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    // Set process group
    if cfg!(target_os = "linux") {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }

    cmd
}
