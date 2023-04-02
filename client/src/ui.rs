use std::io::Result as IOResult;
use std::sync::{Arc, Mutex};
use tui::backend::Backend;
use tui::style::{Color, Style};
use tui::Terminal;
use tui::widgets::{Block, Borders};
use crate::ClientState;

pub fn render<B>(term: &mut Terminal<B>, state: Arc<Mutex<ClientState>>) -> IOResult<()>  where B : Backend {
    term.draw(|f| {
        let size = f.size();
        let status = &state.lock().unwrap().status;

        let block = Block::default()
            .style(
                Style::default()
                    .bg(Color::Black)
            ).title(format!("{status:?}"))
            .borders(Borders::ALL);
        f.render_widget(block, size);
    })?;

    Ok(())
}