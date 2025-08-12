mod cell;
mod flow;
mod list;
mod text;
mod fixed_measurement;

pub use cell::*;
pub use fixed_measurement::*;
pub use flow::*;
pub use list::*;
pub use text::*;

use crate::frame_ctx::FrameContext;
use crate::UIResult;
use ratatui::layout::Size;

pub trait Component {
    type State: Default + 'static;
    type Output;

    fn render(&self, context: &FrameContext, state: &mut Self::State) -> UIResult<Self::Output>;
}

pub trait MeasurableComponent: Component {
    fn measure(&self, context: &FrameContext, state: &Self::State) -> UIResult<Size>;
}

pub const ATTR_KEY_NAV_DOWN: &'static str = "keybinds.common.nav_down";
pub const ATTR_KEY_NAV_UP: &'static str = "keybinds.common.nav_up";
pub const ATTR_KEY_NAV_LEFT: &'static str = "keybinds.common.nav_left";
pub const ATTR_KEY_NAV_RIGHT: &'static str = "keybinds.common.nav_right";

pub const ATTR_COLOR_HIGHLIGHT: &'static str = "colors.common.highlight";
