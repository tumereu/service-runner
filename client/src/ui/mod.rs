use std::sync::{Arc, Mutex};

use tui::backend::Backend;
use tui::Terminal;

use screens::*;
pub use state::{UIState, ViewProfilePane, ViewProfileState, ViewProfileFloatingPane};

use crate::ui::init::render_init;
use crate::ui::profile_select::render_profile_select;
use crate::ui::screens::view_profile::render_view_profile;
use crate::ClientState;
use crate::ui::screens::exit::render_exit;

mod screens;
mod state;
mod widgets;

pub fn render<B>(term: &mut Terminal<B>, state: Arc<Mutex<ClientState>>) -> std::io::Result<()>
where
    B: Backend,
{
    term.draw(|f| {
        let mut state = state.lock().unwrap();
        let frame_size = f.size();
        state.last_frame_size = (frame_size.width, frame_size.height);
        match &state.ui {
            UIState::Initializing => render_init(f, &state),
            UIState::ProfileSelect { .. } => render_profile_select(f, &state),
            UIState::ViewProfile { .. } => render_view_profile(f, &state),
            UIState::Exiting => render_exit(f, &state),
        }
    })?;

    Ok(())
}
