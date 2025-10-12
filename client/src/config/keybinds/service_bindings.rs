use macros::PartialStruct;
use ui::input::KeyMatcher;
use crate::config::keybinds::keybinding::Keybinding;

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