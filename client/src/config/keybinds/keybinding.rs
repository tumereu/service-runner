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
