use std::sync::{Arc, Mutex};

use crate::ui::screens::select_profile::SelectProfileScreen;
use crate::SystemState;
pub use state::{CurrentScreen, UIState, ViewProfileFloatingPane, ViewProfilePane, ViewProfileState};
use ui::component::Component;
use ui::{FrameContext, RenderArgs, UIResult};

mod legacy_screens;
mod state;
mod widgets;
mod screens;

pub struct ViewRoot {
    pub state: Arc<Mutex<SystemState>>
}
impl Component for ViewRoot {
    type Output = ();

    fn render(&self, context: &mut FrameContext) -> UIResult<Self::Output> {
        let state = self.state.lock().unwrap();

        context.render_component(
            RenderArgs::new(
                &SelectProfileScreen {
                    profiles: &state.config.profiles,
                }
            )
        )?;

        Ok(())
    }
}