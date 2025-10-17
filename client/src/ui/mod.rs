use crate::ui::inputs::ATTR_KEY_QUIT;
use crate::ui::screens::select_profile::SelectProfileScreen;
use crate::ui::screens::view_profile::ViewProfileScreen;
use crate::SystemState;
use std::sync::{Arc, RwLock};
use ui::component::Component;
use ui::input::KeyMatcherQueryable;
use ui::{FrameContext, RenderArgs, UIResult};

mod screens;

pub mod inputs;
pub mod theming;

pub struct ViewRoot<'a> {
    pub system_state: &'a mut SystemState
}
impl<'a> Component for ViewRoot<'a> {
    type Output = ();

    fn render(self, context: &mut FrameContext) -> UIResult<Self::Output> {
        let has_profile = self.system_state.current_profile.is_some();

        if has_profile {
            context.render_component(RenderArgs::new(ViewProfileScreen {
                system_state: self.system_state,
            }))?;
        } else {
            context.render_component(RenderArgs::new(SelectProfileScreen {
                system_state: self.system_state,
            }))?;
        }

        if context
            .signals()
            .is_key_pressed(context.req_attr(ATTR_KEY_QUIT)?)
        {
            self.system_state.should_exit = true;
        }

        Ok(())
    }
}
