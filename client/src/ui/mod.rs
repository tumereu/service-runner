use std::sync::{Arc, Mutex};

use tui::backend::Backend;


use tui::Terminal;



pub use state::UIState;

use crate::ClientState;
use crate::ui::init::render_init;
use crate::ui::profile_select::render_profile_select;

mod state;
mod widgets;
mod screens;

use screens::*;
use crate::ui::screens::view_profile::render_view_profile;

pub fn render<B>(
    term: &mut Terminal<B>,
    state: Arc<Mutex<ClientState>>,
) -> std::io::Result<()> where B : Backend {
    term.draw(|f| {
        let state = state.lock().unwrap();
        match &state.ui {
            UIState::Initializing => render_init(f, &state),
            UIState::ProfileSelect { .. } => render_profile_select(f, &state),
            UIState::ViewProfile { .. } => render_view_profile(f, &state),
        }
    })?;

    Ok(())
}
