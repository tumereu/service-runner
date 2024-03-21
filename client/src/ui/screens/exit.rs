use ratatui::Frame;

use crate::client_state::ClientState;
use crate::ui::widgets::{Align, Cell, Text, Flow, IntoCell, render_root, Spinner};

pub fn render_exit(frame: &mut Frame, _state: &ClientState)
{
    render_root(
        Cell {
            align_vert: Align::Center,
            align_horiz: Align::Center,
            element: Flow {
                cells: vec![
                    Cell {
                        element: Text {
                            text: String::from("The system is exiting"),
                            ..Default::default()
                        }.into_el(),
                        ..Default::default()
                    },
                    Cell {
                        element: Spinner {
                            active: true,
                            ..Default::default()
                        }.into_el(),
                        ..Default::default()
                    }
                ],
                ..Default::default()
            }
                .into_el(),
            ..Default::default()
        },
        frame,
    );
}
