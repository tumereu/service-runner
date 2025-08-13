use std::sync::{Arc, Mutex, RwLock};

use crate::ui::screens::select_profile::SelectProfileScreen;
use crate::SystemState;
pub use state::{CurrentScreen, UIState, ViewProfileFloatingPane, ViewProfilePane, ViewProfileState};
use ui::component::Component;
use ui::{FrameContext, RenderArgs, UIResult};
use crate::ui::actions::ActionStore;
use crate::ui::screens::view_profile::ViewProfileScreen;

mod legacy_screens;
mod state;
mod widgets;
mod screens;
pub mod actions;

pub struct ViewRoot {
    pub state: Arc<RwLock<SystemState>>
}
impl Component for ViewRoot {
    type Output = ();

    fn render(&self, context: &mut FrameContext) -> UIResult<Self::Output> {
        let has_profile = self.state.read().unwrap().current_profile.is_some();
        let state = self.state.read().unwrap();
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

        Ok(())
    }
}