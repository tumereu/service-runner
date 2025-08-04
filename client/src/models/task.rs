use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskId(pub usize);

#[derive(Debug, Clone)]
pub struct Task {
    pub id: TaskId,
    pub definition_id: String,
    pub service_id: Option<String>,
    pub status: TaskStatus,
    pub start_time: Instant,
    pub action: Option<TaskAction>,
}

#[derive(Debug, Clone)]
pub enum TaskStatus {
    Running {
        completed_steps: usize,
    },
    Finished { end_time: Instant },
    Failed { end_time: Instant },
}
impl Default for TaskStatus {
    fn default() -> Self {
        Self::Running { completed_steps: 0 }
    }
}

#[derive(Debug, Clone)]
pub enum TaskAction {
    Cancel
}