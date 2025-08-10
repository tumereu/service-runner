use ratatui::style::{Color, Style};
use ratatui::text::Span;
use ratatui::widgets::Paragraph;
use crate::canvas::FrameContext;
use crate::component::{Component, Measurement};
use crate::signal::Signals;
use crate::space::Size;

#[derive(Debug, Default)]
pub struct Text {
    pub text: String,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
}
impl Text {
    pub fn size(&self) -> Size {
        (self.text.len() as u16, 1u16).into()
    }
}

impl Component for Text {
    type State = ();
    type Output = ();

    fn measure(&self, _context: &FrameContext, _state: &Self::State) -> Measurement {
        Measurement {
            min: Some(self.size()),
            max: Some(self.size()),
        }
    }

    fn render(&self, context: &FrameContext, _state: &mut Self::State) -> Self::Output {
        let style = Style::default()
            .fg(self.fg.unwrap_or(Color::Reset));

        context.render_widget(
            Paragraph::new(Span::styled(self.text.clone(), style)),
            self.size().rect_at_origin(),
        );
    }
}