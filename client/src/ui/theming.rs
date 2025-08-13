use ui::ComponentRenderer;
use crate::config::Theme;

pub trait RegisterTheme {
    fn register_theme(&mut self, theme: &Theme);
}

impl RegisterTheme for ComponentRenderer {
    fn register_theme(&mut self, theme: &Theme) {
        self.set_attr(ATTR_COLOR_WORK_ACTIVE, theme.active_color);
        self.set_attr(ATTR_COLOR_WORK_WAITING_TO_PROCESS, theme.waiting_to_process_color);
        self.set_attr(ATTR_COLOR_WORK_PROCESSING, theme.processing_color);
        self.set_attr(ATTR_COLOR_WORK_ERROR, theme.error_color);
        self.set_attr(ATTR_COLOR_WORK_INACTIVE, theme.inactive_color);
        self.set_attr(ATTR_COLOR_WORK_IDLE, theme.idle_color);
        
        self.set_attr(ATTR_COLOR_FOCUSED_ELEMENT, theme.focused_element);
        self.set_attr(ATTR_COLOR_UNFOCUSED_ELEMENT, theme.unfocused_element);
    }
}

pub const ATTR_COLOR_FOCUSED_ELEMENT : &'static str = "colors.common.focused_element";
pub const ATTR_COLOR_UNFOCUSED_ELEMENT : &'static str = "colors.common.unfocused_element";

pub const ATTR_COLOR_WORK_ACTIVE : &'static str = "colors.work.active";
pub const ATTR_COLOR_WORK_WAITING_TO_PROCESS : &'static str = "colors.work.waiting_to_process";
pub const ATTR_COLOR_WORK_PROCESSING : &'static str = "colors.work.processing";
pub const ATTR_COLOR_WORK_ERROR : &'static str = "colors.work.error";
pub const ATTR_COLOR_WORK_INACTIVE : &'static str = "colors.work.inactive";
pub const ATTR_COLOR_WORK_IDLE : &'static str = "colors.work.inactive";
