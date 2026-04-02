use crate::config::Theme;
use ratatui::style::Color;
use ui::{AttrKey, ComponentRenderer};

pub trait RegisterTheme {
    fn register_theme(&mut self, theme: &Theme);
}

impl RegisterTheme for ComponentRenderer {
    fn register_theme(&mut self, theme: &Theme) {
        self.set_attr(ATTR_COLOR_WORK_ACTIVE, theme.active_color.0);
        self.set_attr(
            ATTR_COLOR_WORK_PARTIALLY_ACTIVE,
            theme.partially_active_color.0,
        );
        self.set_attr(
            ATTR_COLOR_WORK_WAITING_TO_PROCESS,
            theme.waiting_to_process_color.0,
        );
        self.set_attr(ATTR_COLOR_WORK_PROCESSING, theme.processing_color.0);
        self.set_attr(ATTR_COLOR_WORK_ERROR, theme.error_color.0);
        self.set_attr(ATTR_COLOR_WORK_INACTIVE, theme.inactive_color.0);
        self.set_attr(ATTR_COLOR_WORK_IDLE, theme.idle_color.0);

        self.set_attr(ATTR_COLOR_FOCUSED_ELEMENT, theme.focused_element.0);
        self.set_attr(ATTR_COLOR_UNFOCUSED_ELEMENT, theme.unfocused_element.0);
    }
}

pub const ATTR_COLOR_FOCUSED_ELEMENT: AttrKey<Color> = AttrKey::new("colors.common.focused_element");
pub const ATTR_COLOR_UNFOCUSED_ELEMENT: AttrKey<Color> = AttrKey::new("colors.common.unfocused_element");

pub const ATTR_COLOR_WORK_ACTIVE: AttrKey<Color> = AttrKey::new("colors.work.active");
pub const ATTR_COLOR_WORK_PARTIALLY_ACTIVE: AttrKey<Color> = AttrKey::new("colors.work.partially_active");
pub const ATTR_COLOR_WORK_WAITING_TO_PROCESS: AttrKey<Color> = AttrKey::new("colors.work.waiting_to_process");
pub const ATTR_COLOR_WORK_PROCESSING: AttrKey<Color> = AttrKey::new("colors.work.processing");
pub const ATTR_COLOR_WORK_ERROR: AttrKey<Color> = AttrKey::new("colors.work.error");
pub const ATTR_COLOR_WORK_INACTIVE: AttrKey<Color> = AttrKey::new("colors.work.inactive");
pub const ATTR_COLOR_WORK_IDLE: AttrKey<Color> = AttrKey::new("colors.work.inactive");
