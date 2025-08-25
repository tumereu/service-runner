use std::collections::HashMap;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::config::{Block, BlockId, ServiceDefinition};
use crate::models::Automation;

#[derive(Debug, Clone)]
pub struct Service {
    pub definition: ServiceDefinition,
    block_statuses: HashMap<BlockId, BlockStatus>,
    block_actions: HashMap<BlockId, BlockAction>,
    pub automations: Vec<Automation>,
    pub output_enabled: bool,
    pub automation_enabled: bool,
}
impl Service {
    pub fn update_block_status(&mut self, block_id: &BlockId, status: BlockStatus)
    {
        if self.definition.blocks.iter().all(|block| &block.id != block_id) {
            return;
        }

        self.block_statuses.insert(block_id.clone(), status);
    }

    pub fn get_block_status(&self, block_id: &BlockId) -> BlockStatus
    {
        self.block_statuses.get(block_id).unwrap_or(&BlockStatus::Initial).clone()
    }

    pub fn update_block_action(&mut self, block_id: &BlockId, action: Option<BlockAction>)
    {
        if self.definition.blocks.iter().all(|block| &block.id != block_id) {
            return;
        }

        match action {
            Some(action) => self.block_actions.insert(block_id.to_owned(), action),
            None => self.block_actions.remove(block_id),
        };
    }

    pub fn get_block_action(&self, block_id: &BlockId) -> Option<BlockAction>
    {
        self.block_actions.get(block_id).map(|action| action.clone())
    }
}
impl From<ServiceDefinition> for Service {
    fn from(value: ServiceDefinition) -> Self {
        Service {
            block_statuses: HashMap::new(),
            block_actions: value.blocks.iter()
                .map(|block| (block.id.clone(), BlockAction::Run))
                .collect(),
            automations: value.automation.iter()
                .map(|auto_def| (auto_def.clone(), Some(value.id.clone())).into())
                .collect(),
            definition: value,
            output_enabled: true,
            automation_enabled: true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum BlockStatus {
    Initial,
    Working {
        step: WorkStep,
    },
    Ok { 
        // If `false`, then the block bypassed its work-steps due to a pre-work health check succeeding. This is only
        // possible for some types of blocks. `true` otherwise.
        was_worked: bool 
    },
    Error,
    Disabled,
}

#[derive(Debug, Clone)]
pub enum WorkStep {
    ResourceGroupCheck {
        /// If `true`, then the actual work step will be skipped if the block is deemed healthy
        /// before execution. If `false`, then pre-work health checks will not be performed and work
        /// is always performed. Has no effect if the block is a non-detatched process -- such blocks must always be
        /// executed.
        skip_work_if_healthy: bool,
    },
    PrerequisiteCheck {
        /// If `true`, then the actual work step will be skipped if the block is deemed healthy
        /// before execution. If `false`, then pre-work health checks will not be performed and work
        /// is always performed. Has no effect if the block is a non-detatched process -- such blocks must always be
        /// executed.
        skip_work_if_healthy: bool,
        start_time: Instant,
        checks_completed: usize,
        last_failure: Option<Instant>,
    },
    PreWorkHealthCheck {
        start_time: Instant,
        checks_completed: usize,
    },
    PerformWork {
        current_step_started: Instant,
        steps_completed: usize,
    },
    PostWorkHealthCheck {
        start_time: Instant,
        checks_completed: usize,
        last_failure: Option<Instant>,
    },
}
impl WorkStep {
    pub fn initial(skip_work_if_healthy: bool) -> Self {
        Self::ResourceGroupCheck { skip_work_if_healthy }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BlockAction {
    #[serde(rename = "disable")]
    Disable,
    #[serde(rename = "enable")]
    Enable,
    #[serde(rename = "toggle_enabled")]
    ToggleEnabled,
    #[serde(rename = "run")]
    Run,
    #[serde(rename = "rerun")]
    ReRun,
    #[serde(rename = "stop")]
    Stop,
    #[serde(rename = "cancel")]
    Cancel,
}

pub trait GetBlock {
    fn get_block(&self, block_id: &BlockId) -> Option<&Block>;
}

impl GetBlock for ServiceDefinition {
    fn get_block(&self, block_id: &BlockId) -> Option<&Block> {
        self.blocks.iter().find(|block| &block.id == block_id)
    }
}
impl GetBlock for Service {
    fn get_block(&self, block_id: &BlockId) -> Option<&Block> {
        self.definition.get_block(block_id)
    }
}