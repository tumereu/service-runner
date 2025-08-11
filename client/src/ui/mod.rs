use std::sync::{Arc, Mutex};

pub use state::{CurrentScreen, UIState, ViewProfileFloatingPane, ViewProfilePane, ViewProfileState};
use ui::{FrameContext, RenderArgs};
use ui::component::{Component};
use crate::SystemState;
use crate::ui::screens::select_profile::SelectProfileScreen;

mod legacy_screens;
mod state;
mod widgets;
mod screens;

pub struct ViewRoot {
    pub state: Arc<Mutex<SystemState>>
}
impl Component for ViewRoot {
    type State = ();
    type Output = ();

    fn render(&self, context: &FrameContext, _state: &mut Self::State) -> Self::Output {
        context.render_component(
            RenderArgs::new(
                &SelectProfileScreen {

                }
            ).key("select-profile")
        );
    }
}