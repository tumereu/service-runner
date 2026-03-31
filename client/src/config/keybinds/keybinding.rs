use serde_derive::{Deserialize, Serialize};
use ui::input::KeyMatcher;

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

impl From<KeyMatcher> for Keybinding {
    fn from(val: KeyMatcher) -> Self {
        Keybinding::Single(val)
    }
}

impl<const L: usize> From<[KeyMatcher; L]> for Keybinding {
    fn from(val: [KeyMatcher; L]) -> Self {
        Keybinding::Multi(Vec::from(val))
    }
}
