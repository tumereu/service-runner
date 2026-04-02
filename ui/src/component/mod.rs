mod cell;
mod fixed_measurement;
mod flow;
mod list;
mod simple_list;
mod space;
mod spinner;
mod text;

pub use cell::*;
pub use fixed_measurement::*;
pub use flow::*;
pub use list::*;
pub use simple_list::*;
pub use space::*;
pub use spinner::*;
pub use text::*;

use crate::UIResult;
use crate::attr_key::AttrKey;
use crate::frame_ctx::FrameContext;
use crate::input::KeyMatcher;
use ratatui::layout::Size;
use ratatui::style::Color;

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

impl<T, S: StatefulComponent<State = T>> Component for S
where
    T: Default + 'static,
{
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

pub const ATTR_KEY_NAV_DOWN: AttrKey<Vec<KeyMatcher>> = AttrKey::new("keybinds.common.nav_down");
pub const ATTR_KEY_NAV_UP: AttrKey<Vec<KeyMatcher>> = AttrKey::new("keybinds.common.nav_up");
pub const ATTR_KEY_NAV_LEFT: AttrKey<Vec<KeyMatcher>> = AttrKey::new("keybinds.common.nav_left");
pub const ATTR_KEY_NAV_RIGHT: AttrKey<Vec<KeyMatcher>> = AttrKey::new("keybinds.common.nav_right");

pub const ATTR_KEY_NAV_DOWN_LARGE: AttrKey<Vec<KeyMatcher>> = AttrKey::new("keybinds.common.nav_down_large");
pub const ATTR_KEY_NAV_UP_LARGE: AttrKey<Vec<KeyMatcher>> = AttrKey::new("keybinds.common.nav_up_large");
pub const ATTR_KEY_NAV_LEFT_LARGE: AttrKey<Vec<KeyMatcher>> = AttrKey::new("keybinds.common.nav_left_large");
pub const ATTR_KEY_NAV_RIGHT_LARGE: AttrKey<Vec<KeyMatcher>> = AttrKey::new("keybinds.common.nav_right_large");

pub const ATTR_KEY_NAV_TO_START: AttrKey<Vec<KeyMatcher>> = AttrKey::new("keybinds.common.nav_to_start");
pub const ATTR_KEY_NAV_TO_END: AttrKey<Vec<KeyMatcher>> = AttrKey::new("keybinds.common.nav_to_end");

pub const ATTR_KEY_CANCEL: AttrKey<Vec<KeyMatcher>> = AttrKey::new("keybinds.common.cancel");
pub const ATTR_KEY_SELECT: AttrKey<Vec<KeyMatcher>> = AttrKey::new("keybinds.common.select");

pub const ATTR_COLOR_HIGHLIGHT: AttrKey<Color> = AttrKey::new("colors.common.highlight");
