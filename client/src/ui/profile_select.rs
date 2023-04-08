use std::sync::{Arc, Mutex};
use tui::backend::Backend;
use tui::{Frame, Terminal};
use crate::client_state::ClientState;

pub fn render_profile_select<B>(
    frame: &mut Frame<B>,
    state: Arc<Mutex<ClientState>>,
) where B : Backend {
}
