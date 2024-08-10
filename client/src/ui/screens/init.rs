use tui::backend::Backend;
use tui::text::Spans;
use tui::widgets::Paragraph;
use tui::Frame;

use crate::system_state::SystemState;

pub fn render_init<B>(frame: &mut Frame<B>, _state: &SystemState)
where
    B: Backend,
{
    let size = frame.size();
    frame.render_widget(
        Paragraph::new(vec![
            Spans::from("Establishing connection to server"),
            Spans::from("Please wait"),
        ]),
        size,
    )
}
