use std::sync::{Arc, Mutex};

use tui::{Frame, Terminal};
use tui::backend::Backend;

use crate::client_state::ClientState;

pub fn render_init<B>(
    frame: &mut Frame<B>,
    state: Arc<Mutex<ClientState>>,
) where B : Backend {
}
