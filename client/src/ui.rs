use std::io::Result as IOResult;
use std::sync::{Arc, Mutex};

use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::Terminal;
use tui::widgets::{Block, Borders, List, ListItem};

use shared::config::Config;

use crate::ClientState;

pub fn render<B>(
    term: &mut Terminal<B>,
    state: Arc<ClientState>,
) -> IOResult<()>  where B : Backend {
    term.draw(|f| {
        let size = f.size();
        let status = state.status;
        let num_services = state.config.services.len();

        let block = Block::default()
            .style(
                Style::default()
                    .bg(Color::Black)
            ).title(format!("{status:?} {num_services}"))
            .borders(Borders::ALL);
        f.render_widget(block, size);

        let list = List::new(
            config.services.iter()
                .map(|service| {
                    ListItem::new(service.name().clone())
                }).collect::<Vec<ListItem>>()
        );

        f.render_widget(list, Rect::new(10, 10, 20, 20));
    })?;

    Ok(())
}