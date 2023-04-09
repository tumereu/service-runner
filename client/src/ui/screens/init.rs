use std::sync::{Arc, Mutex};

use tui::{Frame, Terminal};
use tui::backend::Backend;
use tui::text::{Span, Spans};
use tui::widgets::Paragraph;

use crate::client_state::ClientState;

pub fn render_init<B>(
    frame: &mut Frame<B>,
    state: &ClientState,
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
