use tui::backend::Backend;
use tui::Frame;
use crate::client_state::ClientState;
use crate::ui::UIState;
use crate::ui::widgets::{Flex, FlexAlign, FlexElement, List, render_root};

pub fn render_view_profile<B>(
    frame: &mut Frame<B>,
    state: &ClientState,
) where B : Backend {
    match &state.ui {
        UIState::ViewProfile { } => {},
        any @ _ => panic!("Invalid UI state in render_profile_select: {any:?}")
    };

    render_root(
        Flex::new()
            .children(
                vec![
                    FlexElement {
                        align_vert: FlexAlign::Center,
                        align_horiz: FlexAlign::Center,
                        ..FlexElement::from(
                            List::new().items(vec![String::from("WOW")])
                        )
                    }
                ]
            ),
        frame
    );
}
