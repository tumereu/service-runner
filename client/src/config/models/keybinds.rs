use crate::models::BlockAction;
use crossterm::event::KeyCode;
use serde_derive::{Deserialize, Serialize};
use ui::input::KeyMatcher;

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(default)]
pub struct Keybinds {
    pub common: CommonKeybindings,
    pub output: OutputBindings,
    pub service: ServiceBindings,
    pub block_actions: Vec<ServiceActionBinding>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct CommonKeybindings {
    pub quit: Option<Keybinding>,
    pub focus_next: Option<Keybinding>,
    pub focus_prev: Option<Keybinding>,
    pub nav_left: Option<Keybinding>,
    pub nav_right: Option<Keybinding>,
    pub nav_up: Option<Keybinding>,
    pub nav_down: Option<Keybinding>,
    pub nav_left_large: Option<Keybinding>,
    pub nav_right_large: Option<Keybinding>,
    pub nav_up_large: Option<Keybinding>,
    pub nav_down_large: Option<Keybinding>,
    pub nav_to_start: Option<Keybinding>,
    pub nav_to_end: Option<Keybinding>,
    pub select: Option<Keybinding>,
    pub cancel: Option<Keybinding>,
}
impl Default for CommonKeybindings {
    fn default() -> Self {
        Self {
            quit: KeyMatcher::char('q').ctrl().to_binding(),
            focus_next: KeyMatcher::new(KeyCode::Tab).to_binding(),
            focus_prev: KeyMatcher::new(KeyCode::Tab).shift().to_binding(),
            nav_left: [KeyMatcher::char('h'), KeyMatcher::new(KeyCode::Left)].to_binding(),
            nav_left_large: [
                KeyMatcher::char('h').ctrl(),
                KeyMatcher::new(KeyCode::Left).ctrl(),
                KeyMatcher::char('h').shift(),
                KeyMatcher::new(KeyCode::Left).shift(),
            ]
            .to_binding(),
            nav_right: [KeyMatcher::char('l'), KeyMatcher::new(KeyCode::Right)].to_binding(),
            nav_right_large: [
                KeyMatcher::char('l').ctrl(),
                KeyMatcher::new(KeyCode::Right).ctrl(),
                KeyMatcher::char('l').shift(),
                KeyMatcher::new(KeyCode::Right).shift(),
            ]
            .to_binding(),
            nav_up: [KeyMatcher::char('k'), KeyMatcher::new(KeyCode::Up)].to_binding(),
            nav_up_large: [
                KeyMatcher::char('k').ctrl(),
                KeyMatcher::char('k').shift(),
                KeyMatcher::new(KeyCode::Up).ctrl(),
                KeyMatcher::new(KeyCode::Up).shift(),
                KeyMatcher::new(KeyCode::PageUp),
            ]
            .to_binding(),
            nav_down: [KeyMatcher::char('j'), KeyMatcher::new(KeyCode::Down)].to_binding(),
            nav_down_large: [
                KeyMatcher::char('j').ctrl(),
                KeyMatcher::char('j').shift(),
                KeyMatcher::new(KeyCode::Down).ctrl(),
                KeyMatcher::new(KeyCode::Down).shift(),
                KeyMatcher::new(KeyCode::PageDown),
            ]
            .to_binding(),
            nav_to_start: KeyMatcher::char('g').to_binding(),
            nav_to_end: KeyMatcher::char('g').shift().to_binding(),
            select: [KeyMatcher::char(' '), KeyMatcher::new(KeyCode::Enter)].to_binding(),
            cancel: KeyMatcher::new(KeyCode::Esc).to_binding(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct OutputBindings {
    pub toggle_wrap: Option<Keybinding>,
}
impl Default for OutputBindings {
    fn default() -> Self {
        Self {
            toggle_wrap: KeyMatcher::char('w').to_binding(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct ServiceBindings {
    pub toggle_output_selected: Option<Keybinding>,
    pub toggle_output_all: Option<Keybinding>,
    pub toggle_automation_selected: Option<Keybinding>,
    pub toggle_automation_all: Option<Keybinding>,
}
impl Default for ServiceBindings {
    fn default() -> Self {
        Self {
            toggle_output_selected: KeyMatcher::char('o').to_binding(),
            toggle_output_all: KeyMatcher::char('o').shift().to_binding(),
            toggle_automation_selected: KeyMatcher::char('a').to_binding(),
            toggle_automation_all: KeyMatcher::char('a').shift().to_binding(),
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
impl Into<Vec<KeyMatcher>> for Keybinding {
    fn into(self) -> Vec<KeyMatcher> {
        match self {
            Keybinding::Single(binding) => vec![binding],
            Keybinding::Multi(bindings) => bindings,
        }
    }
}

trait IntoKeybinding {
    fn to_binding(self) -> Option<Keybinding>;
}
impl IntoKeybinding for KeyMatcher {
    fn to_binding(self) -> Option<Keybinding> {
        Some(Keybinding::Single(self))
    }
}
impl<const L: usize> IntoKeybinding for [KeyMatcher; L] {
    fn to_binding(self) -> Option<Keybinding> {
        Some(Keybinding::Multi(Vec::from(self)))
    }
}
