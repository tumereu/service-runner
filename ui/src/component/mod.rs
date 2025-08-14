mod cell;
mod flow;
mod list;
mod text;
mod fixed_measurement;
mod space;
mod spinner;

pub use cell::*;
pub use fixed_measurement::*;
pub use flow::*;
pub use list::*;
pub use text::*;
pub use space::*;
pub use spinner::*;

use crate::frame_ctx::FrameContext;
use crate::UIResult;
use ratatui::layout::Size;

pub trait Component {
    type Output;

    fn render(self, context: &mut FrameContext) -> UIResult<Self::Output>;
}

pub trait StatefulComponent {
    type Output;
    type State;

    fn state_id(&self) -> &str;
    fn render(self, context: &mut FrameContext, state: &mut Self::State) -> UIResult<Self::Output>;
}

impl<T, S : StatefulComponent<State = T>> Component for S where T : Default + 'static {
    type Output = S::Output;

    fn render(self, context: &mut FrameContext) -> UIResult<Self::Output> {
        let state_id = self.state_id().to_owned();
        let mut state = context.take_state::<T>(&state_id);
        let result = self.render(context, &mut state);
        context.return_state(&state_id, state);
        result
    }
}

pub trait MeasurableComponent: Component {
    fn measure(&self, context: &FrameContext) -> UIResult<Size>;
}

pub const ATTR_KEY_NAV_DOWN: &'static str = "keybinds.common.nav_down";
pub const ATTR_KEY_NAV_UP: &'static str = "keybinds.common.nav_up";
pub const ATTR_KEY_NAV_LEFT: &'static str = "keybinds.common.nav_left";
pub const ATTR_KEY_NAV_RIGHT: &'static str = "keybinds.common.nav_right";

pub const ATTR_KEY_NAV_DOWN_LARGE: &'static str = "keybinds.common.nav_down_large";
pub const ATTR_KEY_NAV_UP_LARGE: &'static str = "keybinds.common.nav_up_large";
pub const ATTR_KEY_NAV_LEFT_LARGE: &'static str = "keybinds.common.nav_left_large";
pub const ATTR_KEY_NAV_RIGHT_LARGE: &'static str = "keybinds.common.nav_right_large";

pub const ATTR_KEY_NAV_TO_START: &'static str = "keybinds.common.nav_to_start";
pub const ATTR_KEY_NAV_TO_END: &'static str = "keybinds.common.nav_to_end";

pub const ATTR_KEY_CANCEL: &'static str = "keybinds.common.cancel";
pub const ATTR_KEY_SELECT: &'static str = "keybinds.common.select";

pub const ATTR_COLOR_HIGHLIGHT: &'static str = "colors.common.highlight";
