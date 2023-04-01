use tui::backend::Backend;
use tui::style::{Color, Style};
use tui::Terminal;
use tui::widgets::{Block, Borders};
use crate::AppState;

pub fn render<B>(term: &mut Terminal<B>, app_state: &AppState) where B : Backend {
    term.draw(|f| {
        let size = f.size();

        let block = Block::default()
            .style(
                Style::default()
                    .bg(Color::Black)
            ).title(format!("Block"))
            .borders(Borders::ALL);
        f.render_widget(block, size);
    });
}