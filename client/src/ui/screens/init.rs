use ratatui::backend::Backend;
use ratatui::Frame;

use crate::client_state::ClientState;

pub fn render_init(frame: &mut Frame, _state: &ClientState)
{
    let size = frame.size();
    // TODO render a loading screen?
}
