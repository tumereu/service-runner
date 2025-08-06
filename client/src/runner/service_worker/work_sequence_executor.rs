use reqwest::blocking::Client as HttpClient;
use reqwest::Method;
use std::net::{TcpListener};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use log::debug;
use crate::config::{ExecutableEntry, HttpMethod, Requirement, TaskStep};
use crate::runner::rhai::RHAI_ENGINE;
use crate::runner::service_worker::{ConcurrentOperationStatus, WorkResult};
use crate::runner::service_worker::requirement_checker::{RequirementCheckResult, RequirementChecker};
use crate::runner::service_worker::work_context::WorkContext;
use crate::system_state::OperationType;
use crate::utils::format_err;

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
            Some(WorkSequenceEntry::RhaiScript(script)) => self.handle_rhai_script(script),
            Some(WorkSequenceEntry::WaitRequirement { timeout, requirement }) => self.handle_requirement(timeout, requirement),
        }
    }

    fn handle_executable_entry(&self, entry: &ExecutableEntry) -> WorkExecutionResult {
        match self.context.get_concurrent_operation_status() {
            None => {
                let mut command = create_cmd(entry, Some(self.workdir.clone()));
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

    fn handle_rhai_script(&self, script: &String) -> WorkExecutionResult {
        let (result, messages) = {
            // TODO currently evaluation is performed synchronously. Move engine to a worker thread to allow for
            //      longer scripts?
            let mut scope = self.context.create_rhai_scope();
            match RHAI_ENGINE.eval_with_scope::<bool>(&mut scope, script) {
                Ok(_) => (
                    WorkExecutionResult::EntryOk,
                    vec![
                        format!("Rhai script OK: {}", script)
                    ]
                ),
                Err(error) => (
                    WorkExecutionResult::Failed,
                    vec![
                        format!("Error in Rhai script {script}: {error:?}")
                    ],
                ),
            }
        };

        messages.into_iter().for_each(|message| {
            self.context.add_system_output(message);
        });

        result
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
