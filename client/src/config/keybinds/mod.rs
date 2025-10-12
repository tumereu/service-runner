use serde_derive::Deserialize;
use block_bindings::ServiceActionBinding;
use ui::input::KeyMatcher;

pub use common_bindings::*;
pub use output_bindings::*;
pub use service_bindings::*;
pub use block_bindings::*;
pub use keybinding::*;

mod common_bindings;
mod output_bindings;
mod service_bindings;
mod block_bindings;
mod keybinding;

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