use std::sync::{Arc, Mutex, RwLock};
use log::debug;
use crate::SystemState;
use crate::ui::inputs::ATTR_KEY_QUIT;
use crate::ui::screens::select_profile::SelectProfileScreen;
use crate::ui::screens::view_profile::ViewProfileScreen;
use ui::component::Component;
use ui::input::KeyMatcherQueryable;
use ui::{FrameContext, RenderArgs, UIResult};

mod screens;

pub mod inputs;
pub mod theming;

pub struct ViewRoot {
    pub system_state: Arc<RwLock<SystemState>>,
}
impl Component for ViewRoot {
    type Output = ();

    fn render(self, context: &mut FrameContext) -> UIResult<Self::Output> {
        let has_profile = self.system_state.read().unwrap().current_profile.is_some();

        if has_profile {
            context.render_component(RenderArgs::new(ViewProfileScreen {
                system_state: self.system_state.clone(),
            }))?;
        } else {
            context.render_component(RenderArgs::new(SelectProfileScreen {
                system_state: self.system_state.clone(),
            }))?;
        }

        if context
            .signals()
            .is_key_pressed(context.req_attr(ATTR_KEY_QUIT)?)
        {
            let mut state = self.system_state.write().unwrap();
            state.should_exit = true;
        }

        Ok(())
    }
}
