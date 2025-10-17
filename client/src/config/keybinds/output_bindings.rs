use macros::PartialStruct;
use ui::input::KeyMatcher;
use crate::config::keybinds::keybinding::Keybinding;

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