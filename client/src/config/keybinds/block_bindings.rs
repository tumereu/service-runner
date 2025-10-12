use serde_derive::{Deserialize, Serialize};
use ui::input::KeyMatcher;
use crate::config::keybinds::keybinding::Keybinding;
use crate::models::BlockAction;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ServiceActionBinding {
    pub action: BlockAction,
    #[serde(flatten)]
    pub blocks: ServiceActionBlocks,
    pub target: ServiceActionTarget,
    pub key: Keybinding,
}

impl ServiceActionBinding {
    pub fn resolve(&self) -> ResolvedBlockActionBinding {
        ResolvedBlockActionBinding {
            action: self.action.clone(),
            blocks: match &self.blocks {
                ServiceActionBlocks::Block { block } => vec![block.clone()],
                ServiceActionBlocks::Blocks { blocks } => blocks.clone(),
            },
            target: self.target.clone(),
            keys: match &self.key {
                Keybinding::Single(matcher) => vec![matcher.clone()],
                Keybinding::Multi(matchers) => matchers.clone(),
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum ServiceActionBlocks {
    Block { block: String },
    Blocks { blocks: Vec<String> },
}

#[derive(Debug, Clone)]
pub struct ResolvedBlockActionBinding {
    pub action: BlockAction,
    pub blocks: Vec<String>,
    pub target: ServiceActionTarget,
    pub keys: Vec<KeyMatcher>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ServiceActionTarget {
    #[serde(rename = "selected")]
    Selected,
    #[serde(rename = "all")]
    All,
}