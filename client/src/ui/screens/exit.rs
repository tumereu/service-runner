use tui::backend::Backend;
use tui::Frame;

use crate::system_state::SystemState;
use crate::ui::widgets::{Align, Cell, Text, Flow, IntoCell, render_root, Spinner};

pub fn render_exit<B>(frame: &mut Frame<B>, _state: &SystemState)
where
    B: Backend,
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
