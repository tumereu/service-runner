use std::sync::{Arc, Mutex, RwLock};

use crate::ui::screens::select_profile::SelectProfileScreen;
use crate::SystemState;
pub use state::{CurrentScreen, UIState, ViewProfileFloatingPane, ViewProfilePane, ViewProfileState};
use ui::component::Component;
use ui::{ComponentRenderer, FrameContext, RenderArgs, UIResult};
use ui::input::KeyMatcherQueryable;
use crate::ui::actions::ActionStore;
use crate::ui::inputs::ATTR_KEY_QUIT;
use crate::ui::screens::view_profile::ViewProfileScreen;

mod state;
mod screens;

pub mod theming;
pub mod actions;
pub mod inputs;

pub struct ViewRoot {
    pub state: Arc<RwLock<SystemState>>
}
impl Component for ViewRoot {
    type Output = ();

    fn render(&self, context: &mut FrameContext) -> UIResult<Self::Output> {
        let actions = {
            let state = self.state.read().unwrap();
            let has_profile = state.current_profile.is_some();
            let actions = ActionStore::new();

            if has_profile {
                context.render_component(
                    RenderArgs::new(
                        &ViewProfileScreen {
                            state: &state,
                            actions: &actions
                        }
                    )
                )?;
            } else {
                context.render_component(
                    RenderArgs::new(
                        &SelectProfileScreen {
                            state: &state,
                            actions: &actions
                        }
                    )
                )?;
            }

            actions
        };

        let mut state = self.state.write().unwrap();
        actions.process(&mut state);

        if context.signals().is_key_pressed(context.req_attr(ATTR_KEY_QUIT)?) {
            state.should_exit = true;
        }

        Ok(())
    }
}