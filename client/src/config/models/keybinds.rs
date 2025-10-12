use crate::models::BlockAction;
use crossterm::event::KeyCode;
use serde_derive::{Deserialize, Serialize};
use macros::PartialStruct;
use ui::input::KeyMatcher;

#[derive(Debug, Clone, Default)]
pub struct Keybinds {
    pub common: CommonKeybindings,
    pub output: OutputBindings,
    pub service: ServiceBindings,
    pub block_actions: Vec<ServiceActionBinding>,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct PartialKeybinds {
    pub common: PartialCommonKeybindings,
    pub output: PartialOutputBindings,
    pub service: PartialServiceBindings,
    pub block_actions: Option<Vec<ServiceActionBinding>>,
}
impl PartialKeybinds {
    pub fn apply_to(self, binds: &mut Keybinds) {
        self.common.apply_to(&mut binds.common);
        self.output.apply_to(&mut binds.output);
        self.service.apply_to(&mut binds.service);
        if let Some(block_actions) = self.block_actions {
            binds.block_actions = block_actions;
        }
    }
}

#[derive(Debug, Clone, PartialStruct)]
pub struct CommonKeybindings {
    pub quit: Keybinding,
    pub focus_next: Keybinding,
    pub focus_prev: Keybinding,
    pub nav_left: Keybinding,
    pub nav_right: Keybinding,
    pub nav_up: Keybinding,
    pub nav_down: Keybinding,
    pub nav_left_large: Keybinding,
    pub nav_right_large: Keybinding,
    pub nav_up_large: Keybinding,
    pub nav_down_large: Keybinding,
    pub nav_to_start: Keybinding,
    pub nav_to_end: Keybinding,
    pub select: Keybinding,
    pub cancel: Keybinding,
}
impl Default for CommonKeybindings {
    fn default() -> Self {
        Self {
            quit: KeyMatcher::char('q').ctrl().into(),
            focus_next: KeyMatcher::new(KeyCode::Tab).into(),
            focus_prev: KeyMatcher::new(KeyCode::Tab).shift().into(),
            nav_left: [KeyMatcher::char('h'), KeyMatcher::new(KeyCode::Left)].into(),
            nav_left_large: [
                KeyMatcher::char('h').ctrl(),
                KeyMatcher::new(KeyCode::Left).ctrl(),
                KeyMatcher::char('h').shift(),
                KeyMatcher::new(KeyCode::Left).shift(),
            ]
            .into(),
            nav_right: [KeyMatcher::char('l'), KeyMatcher::new(KeyCode::Right)].into(),
            nav_right_large: [
                KeyMatcher::char('l').ctrl(),
                KeyMatcher::new(KeyCode::Right).ctrl(),
                KeyMatcher::char('l').shift(),
                KeyMatcher::new(KeyCode::Right).shift(),
            ]
            .into(),
            nav_up: [KeyMatcher::char('k'), KeyMatcher::new(KeyCode::Up)].into(),
            nav_up_large: [
                KeyMatcher::char('k').ctrl(),
                KeyMatcher::char('k').shift(),
                KeyMatcher::new(KeyCode::Up).ctrl(),
                KeyMatcher::new(KeyCode::Up).shift(),
                KeyMatcher::new(KeyCode::PageUp),
            ]
            .into(),
            nav_down: [KeyMatcher::char('j'), KeyMatcher::new(KeyCode::Down)].into(),
            nav_down_large: [
                KeyMatcher::char('j').ctrl(),
                KeyMatcher::char('j').shift(),
                KeyMatcher::new(KeyCode::Down).ctrl(),
                KeyMatcher::new(KeyCode::Down).shift(),
                KeyMatcher::new(KeyCode::PageDown),
            ]
            .into(),
            nav_to_start: KeyMatcher::char('g').into(),
            nav_to_end: KeyMatcher::char('g').shift().into(),
            select: [KeyMatcher::char(' '), KeyMatcher::new(KeyCode::Enter)].into(),
            cancel: KeyMatcher::new(KeyCode::Esc).into(),
        }
    }
}

#[derive(Debug, Clone, PartialStruct)]
pub struct OutputBindings {
    pub toggle_wrap: Keybinding,
}
impl Default for OutputBindings {
    fn default() -> Self {
        Self {
            toggle_wrap: KeyMatcher::char('w').into(),
        }
    }
}

#[derive(Debug, Clone, PartialStruct)]
pub struct ServiceBindings {
    pub toggle_output_selected: Keybinding,
    pub toggle_output_all: Keybinding,
    pub toggle_automation_selected: Keybinding,
    pub toggle_automation_all: Keybinding,
}
impl Default for ServiceBindings {
    fn default() -> Self {
        Self {
            toggle_output_selected: KeyMatcher::char('o').into(),
            toggle_output_all: KeyMatcher::char('o').shift().into(),
            toggle_automation_selected: KeyMatcher::char('a').into(),
            toggle_automation_all: KeyMatcher::char('a').shift().into(),
        }
    }
}

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

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum Keybinding {
    Single(KeyMatcher),
    Multi(Vec<KeyMatcher>),
}
impl Keybinding {
    pub fn matchers(&self) -> Vec<KeyMatcher> {
        match self {
            Keybinding::Single(binding) => vec![binding.clone()],
            Keybinding::Multi(bindings) => bindings.clone(),
        }
    }
}
impl Into<Keybinding> for KeyMatcher {
    fn into(self) -> Keybinding {
        Keybinding::Single(self)
    }
}
impl<const L: usize> Into<Keybinding> for [KeyMatcher; L] {
    fn into(self) -> Keybinding {
        Keybinding::Multi(Vec::from(self))
    }
}
