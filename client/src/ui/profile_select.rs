use std::sync::{Arc, Mutex};

use tui::{Frame, Terminal};
use tui::backend::Backend;
use tui::layout::Rect;
use tui::widgets::{List, ListItem};

use crate::client_state::ClientState;

pub fn render_profile_select<B>(
    frame: &mut Frame<B>,
    state: &ClientState,
) where B : Backend {
    let size = frame.size();

    frame.render_widget(
        List::new(
            state.config.services.iter()
                .map(|service| {
                    ListItem::new(service.name().clone())
                }).collect::<Vec<ListItem>>()
        ),
        size
    );
}
