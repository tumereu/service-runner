use std::sync::{Arc, Mutex};

use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::Terminal;
use tui::widgets::{Block, Borders, List, ListItem};

use shared::config::Config;
pub use state::UIState;

use crate::ClientState;
use crate::ui::init::render_init;
use crate::ui::profile_select::render_profile_select;

mod state;
mod init;
mod profile_select;
mod widgets;

pub fn render<B>(
    term: &mut Terminal<B>,
    state: Arc<Mutex<ClientState>>,
) -> std::io::Result<()> where B : Backend {
    term.draw(|f| {
        {
            let state = state.lock().unwrap();
            match &state.ui {
                UIState::Initializing => render_init(f, &state),
                UIState::ProfileSelect { .. } => render_profile_select(f, &state)
            }
        }
    })?;

    Ok(())
}
