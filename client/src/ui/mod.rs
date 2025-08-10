use std::sync::{Arc, Mutex};

pub use state::{CurrentScreen, UIState, ViewProfileFloatingPane, ViewProfilePane, ViewProfileState};
use ui::{render, FrameContext, RenderArgs, RenderContext, Signals};
use ui::component::{Component, Measurement};
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

    fn measure(&self, _context: &FrameContext, _state: &Self::State) -> Measurement {
        Default::default()
    }

    fn render(&self, context: &FrameContext, _state: &mut Self::State) -> Self::Output {
        render!(context, {
            key = "text",
            component = SelectProfileScreen {},
            pos = (0, 0),
        });
    }
}