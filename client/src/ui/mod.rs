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

pub fn render<B>(
    term: &mut Terminal<B>,
    state: Arc<Mutex<ClientState>>,
) -> std::io::Result<()> where B : Backend {
    term.draw(|f| {
        match state.lock().unwrap().ui {
            UIState::Initializing => render_init(f, state.clone()),
            UIState::ProfileSelect { .. } => render_profile_select(f, state.clone())
        }

        let list = List::new(
            state.lock().unwrap().config.services.iter()
                .map(|service| {
                    ListItem::new(service.name().clone())
                }).collect::<Vec<ListItem>>()
        );

        f.render_widget(list, Rect::new(10, 10, 20, 20));
    })?;

    Ok(())
}
