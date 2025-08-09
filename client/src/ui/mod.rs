use std::cell::RefCell;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use ratatui::backend::Backend;
use ratatui::Terminal;

pub use state::{CurrentScreen, UIState, ViewProfileFloatingPane, ViewProfilePane, ViewProfileState};
use ui::{Canvas, RenderArgs, RenderContext};
use ui::component::{Component, Measurement, Text};
use crate::ui::screens::profile_select::render_profile_select;
use crate::ui::screens::view_profile::render_view_profile;
use crate::SystemState;

mod screens;
mod state;
mod widgets;

pub fn render<B>(term: &mut Terminal<B>, system_arc: Arc<Mutex<SystemState>>) -> std::io::Result<()>
where
    B: Backend,
{
    term.draw(|f| {
        let mut state = system_arc.lock().unwrap();
        let frame_size = f.size();
        state.ui.last_frame_size = (frame_size.width, frame_size.height);

        match &state.ui.screen {
            CurrentScreen::ProfileSelect { .. } => render_profile_select(f, &state),
            CurrentScreen::ViewProfile { .. } => render_view_profile(f, &state),
        }
    })?;

    Ok(())
}

pub struct ViewRoot {
    pub state: Arc<Mutex<SystemState>>
}
impl Component<()> for ViewRoot {
    fn measure(&self, _canvas: &Canvas, _ctx: RenderContext<()>) -> Measurement {
        Default::default()
    }

    fn render(&self, canvas: &Canvas, _ctx: RenderContext<()>) {
        canvas.render_component(RenderArgs {
            key: "text".to_string(),
            component: Text {
                text: "123".to_owned(),
                ..Default::default()
            },
            pos: (12, 24).into(),
            size: canvas.size(),
            retain_unmounted_state: false,
            state_type: PhantomData,
        });
    }
}