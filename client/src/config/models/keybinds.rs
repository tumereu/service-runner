use crossterm::event::KeyCode;
use serde_derive::{Deserialize, Serialize};
use ui::input::KeyMatcher;

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(default)]
pub struct Keybinds {
    pub common: CommonKeybindings,
    pub output: OutputBindings,
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
            ]
            .to_binding(),
            nav_right: [KeyMatcher::char('l'), KeyMatcher::new(KeyCode::Right)].to_binding(),
            nav_right_large: [
                KeyMatcher::char('l').ctrl(),
                KeyMatcher::new(KeyCode::Right).ctrl(),
            ]
            .to_binding(),
            nav_up: [KeyMatcher::char('k'), KeyMatcher::new(KeyCode::Up)].to_binding(),
            nav_up_large: [
                KeyMatcher::char('k').ctrl(),
                KeyMatcher::new(KeyCode::Up).ctrl(),
                KeyMatcher::new(KeyCode::PageUp).ctrl(),
            ]
            .to_binding(),
            nav_down: [KeyMatcher::char('j'), KeyMatcher::new(KeyCode::Down)].to_binding(),
            nav_down_large: [
                KeyMatcher::char('j').ctrl(),
                KeyMatcher::new(KeyCode::Down).ctrl(),
                KeyMatcher::new(KeyCode::PageDown).ctrl(),
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
