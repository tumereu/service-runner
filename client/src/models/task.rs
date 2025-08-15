use crate::config::{ServiceId, TaskDefinitionId};
use derive_more::Display;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Display)]
pub struct TaskId(pub usize);

#[derive(Debug, Clone)]
pub struct Task {
    pub id: TaskId,
    pub definition_id: TaskDefinitionId,
    pub service_id: Option<ServiceId>,
    pub status: TaskStatus,
    pub action: Option<TaskAction>,
}

#[derive(Debug, Clone)]
pub enum TaskStatus {
    Running {
        step_start_time: Instant,
        last_recoverable_failure: Option<Instant>,
        completed_steps: usize,
    },
    Finished,
    Failed,
}
impl Default for TaskStatus {
    fn default() -> Self {
        Self::Running { 
            completed_steps: 0,
            step_start_time: Instant::now(),
            last_recoverable_failure: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TaskAction {
    Cancel
}