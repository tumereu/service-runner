use std::sync::{Arc, Mutex};

pub use state::{CurrentScreen, UIState, ViewProfileFloatingPane, ViewProfilePane, ViewProfileState};
use ui::{render, Canvas, RenderArgs, RenderContext};
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

    fn measure(&self, _canvas: &Canvas, _ctx: RenderContext<Self::State>) -> Measurement {
        Default::default()
    }

    fn render(&self, canvas: &Canvas, _ctx: RenderContext<Self::State>) -> Self::Output {
        render!(canvas, {
            key = "text",
            component = SelectProfileScreen {},
            pos = (0, 0),
        });
    }
}