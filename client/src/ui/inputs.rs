use ui::component::{ATTR_KEY_CANCEL, ATTR_KEY_NAV_DOWN, ATTR_KEY_NAV_DOWN_LARGE, ATTR_KEY_NAV_LEFT, ATTR_KEY_NAV_LEFT_LARGE, ATTR_KEY_NAV_RIGHT, ATTR_KEY_NAV_RIGHT_LARGE, ATTR_KEY_NAV_TO_END, ATTR_KEY_NAV_TO_START, ATTR_KEY_NAV_UP, ATTR_KEY_NAV_UP_LARGE, ATTR_KEY_SELECT};
use ui::ComponentRenderer;
use ui::input::KeyMatcher;
use crate::config::{Keybinding, Keybinds, ResolvedBlockActionBinding, ServiceActionTarget};
use crate::models::BlockAction;

pub trait RegisterKeybinds {
    fn register_keybinds(&mut self, binds: &Keybinds);
}

impl RegisterKeybinds for ComponentRenderer {
    fn register_keybinds(&mut self, binds: &Keybinds) {
        binds.common.quit.bind_key(ATTR_KEY_QUIT, self);
        
        binds.common.nav_up.bind_key(ATTR_KEY_NAV_UP, self);
        binds.common.nav_up_large.bind_key(ATTR_KEY_NAV_UP_LARGE, self);
        binds.common.nav_down.bind_key(ATTR_KEY_NAV_DOWN, self);
        binds.common.nav_down_large.bind_key(ATTR_KEY_NAV_DOWN_LARGE, self);
        binds.common.nav_left.bind_key(ATTR_KEY_NAV_LEFT, self);
        binds.common.nav_left_large.bind_key(ATTR_KEY_NAV_LEFT_LARGE, self);
        binds.common.nav_right.bind_key(ATTR_KEY_NAV_RIGHT, self);
        binds.common.nav_right_large.bind_key(ATTR_KEY_NAV_RIGHT_LARGE, self);
        binds.common.nav_to_start.bind_key(ATTR_KEY_NAV_TO_START, self);
        binds.common.nav_to_end.bind_key(ATTR_KEY_NAV_TO_END, self);
        
        binds.common.focus_next.bind_key(ATTR_KEY_FOCUS_NEXT, self);
        binds.common.focus_prev.bind_key(ATTR_KEY_FOCUS_PREV, self);
        
        binds.common.select.bind_key(ATTR_KEY_SELECT, self);
        binds.common.select.bind_key(ATTR_KEY_CANCEL, self);
        
        binds.output.toggle_wrap.bind_key(ATTR_KEY_TOGGLE_WRAP, self);

        binds.service.toggle_output_selected.bind_key(ATTR_KEY_TOGGLE_SELECTED_OUTPUT, self);
        binds.service.toggle_output_all.bind_key(ATTR_KEY_TOGGLE_ALL_OUTPUT, self);

        self.set_attr(ATTR_KEY_BLOCK_ACTIONS, binds.block_actions.iter()
            .map(|action| action.resolve())
            .collect::<Vec<ResolvedBlockActionBinding>>());
    }
}

trait BindKey {
    fn bind_key(&self, attr: &str, target: &mut ComponentRenderer);
}
impl BindKey for Option<Keybinding> {
    fn bind_key(&self, attr: &str, target: &mut ComponentRenderer) {
        let result: Vec<KeyMatcher> = self.as_ref().map(|key| key.clone().into()).unwrap_or_default();
        target.set_attr(attr, result);
    }   
}

pub const ATTR_KEY_QUIT: &'static str = "keybinds.system.quit";

pub const ATTR_KEY_FOCUS_NEXT: &'static str = "keybinds.common.focus_next";
pub const ATTR_KEY_FOCUS_PREV: &'static str = "keybinds.common.focus_prev";

pub const ATTR_KEY_TOGGLE_WRAP: &'static str = "keybinds.text_area.toggle_wrap";

pub const ATTR_KEY_TOGGLE_SELECTED_OUTPUT: &'static str = "keybinds.services.toggle_selected_output";
pub const ATTR_KEY_TOGGLE_ALL_OUTPUT: &'static str = "keybinds.services.toggle_all_output";

pub const ATTR_KEY_BLOCK_ACTIONS: &'static str = "keymappings.service_list.block_actions";
