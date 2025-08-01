use crate::config::{ServiceDefinition, Block};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Service {
    pub definition: ServiceDefinition,
    block_statuses: HashMap<String, BlockStatus>,
    block_actions: HashMap<String, BlockAction>,
    pub enabled: bool,
}
impl Service {
    pub fn update_block_status(&mut self, block_id: &str, status: BlockStatus)
    {
        if self.definition.blocks.iter().all(|block| block.id != block_id) {
            return;
        }
        
        self.block_statuses.insert(block_id.to_owned(), status);
    }

    pub fn get_block_status(&self, block_id: &str) -> BlockStatus
    {
        self.block_statuses.get(block_id).unwrap_or(&BlockStatus::Initial).clone()
    }

    pub fn update_block_action(&mut self, block_id: &str, action: Option<BlockAction>)
    {
        if self.definition.blocks.iter().all(|block| block.id != block_id) {
            return;
        }
        
        match action {
            Some(action) => self.block_actions.insert(block_id.to_owned(), action),
            None => self.block_actions.remove(block_id),
        };
    }

    pub fn get_block_action(&self, block_id: &str) -> Option<BlockAction>
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
            definition: value,
            enabled: true,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BlockStatus {
    Initial,
    Working {
        steps_completed: usize,
        current_step: Option<usize>,
    },
    Ok,
    Error,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BlockAction {
    Disable,
    Enable,
    ToggleEnabled,
    Run,
    ReRun,
    Stop,
    Cancel,
}

pub trait GetBlock {
    fn get_block(&self, block_id: &str) -> Option<&Block>;
}

impl GetBlock for ServiceDefinition {
    fn get_block(&self, block_id: &str) -> Option<&Block> {
        self.blocks.iter().find(|s| s.id == block_id)
    }
}
impl GetBlock for Service {
    fn get_block(&self, block_id: &str) -> Option<&Block> {
        self.definition.get_block(block_id)
    }
}