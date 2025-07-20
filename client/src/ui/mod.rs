use std::sync::{Arc, Mutex};

use tui::backend::Backend;
use tui::Terminal;

pub use state::{CurrentScreen, UIState, ViewProfileFloatingPane, ViewProfilePane, ViewProfileState};

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
