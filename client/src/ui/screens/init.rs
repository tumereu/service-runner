

use tui::{Frame};
use tui::backend::Backend;
use tui::text::{Spans};
use tui::widgets::Paragraph;

use crate::client_state::ClientState;

pub fn render_init<B>(
    frame: &mut Frame<B>,
    _state: &ClientState,
) where B : Backend {
    let size = frame.size();
    frame.render_widget(
        Paragraph::new(vec![
            Spans::from("Establishing connection to server"),
            Spans::from("Please wait")
        ]),
        size
    )
}
