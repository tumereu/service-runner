use crate::models::TaskStatus;
use crate::runner::service_worker::task_context::TaskContext;
use crate::runner::service_worker::work_sequence_executor::{WorkExecutionResult, WorkSequenceEntry, WorkSequenceExecutor};
use log::{debug, error, info};
use std::time::Instant;

pub trait TaskProcessor {
    fn process_task(&self);
}
impl TaskProcessor for TaskContext {
    fn process_task(&self) {
        let task_status = self.get_status();
        let (completed_steps, step_start_time, last_recoverable_failure) = match task_status {
            TaskStatus::Running { completed_steps, step_start_time, last_recoverable_failure } => {
                (completed_steps, step_start_time, last_recoverable_failure)
            },
            TaskStatus::Failed { .. } | TaskStatus::Finished { .. } => {
                error!("System called process_task() but task {task_id}/{def_id} is not running",
                task_id = self.task_id,
                def_id = self.definition_id
            );
                return;
            },
        };


        let service_id = self.query_task(|task| task.service_id.clone());
        let work_seq: Vec<WorkSequenceEntry> = self.query_system(|system| {
            system.get_task_definition(&self.definition_id, service_id)
                .iter().flat_map(|definition| {
                    definition.steps.iter().map(|step| step.clone().into())
                }).collect()
        });
        let workdir = self.query_service(|service| service.definition.workdir.clone());
        let workdir = workdir
            .unwrap_or(self.query_system(|system| system.current_profile.as_ref().unwrap().definition.workdir.clone()));

        let exec_result = WorkSequenceExecutor {
            sequence: work_seq,
            completed_count: completed_steps,
            entry_start_time: step_start_time,
            last_recoverable_failure,
            context: &self.create_work_context(false),
            workdir,
        }.exec_next();

        match exec_result {
            WorkExecutionResult::EntryOk => {
                self.update_status(TaskStatus::Running {
                    step_start_time: Instant::now(),
                    completed_steps: completed_steps + 1,
                    last_recoverable_failure: None,
                });
            },
            WorkExecutionResult::RecoverableFailure => {
                self.update_status(TaskStatus::Running {
                    step_start_time,
                    completed_steps,
                    last_recoverable_failure: Some(Instant::now()),
                });
            }
            WorkExecutionResult::AllOk => {
                self.update_status(TaskStatus::Finished);
            }
            WorkExecutionResult::Failed => {
                self.update_status(TaskStatus::Failed);
            }
            WorkExecutionResult::Working => {
                // Nothing to do but to wait for the task to complete
            }
        }
    }
}
