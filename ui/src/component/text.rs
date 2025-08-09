use ratatui::style::{Color, Style};
use ratatui::text::Span;
use ratatui::widgets::Paragraph;
use crate::canvas::Canvas;
use crate::component::{Component, Measurement};
use crate::space::Size;
use crate::state_store::StoreAccessContext;

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

impl Component<()> for Text {
    fn measure(&self, canvas: &Canvas, state: StoreAccessContext<()>) -> Measurement {
        Measurement {
            min: Some(self.size()),
            max: Some(self.size()),
        }
    }

    fn render(&self, canvas: &Canvas, state: StoreAccessContext<()>) {
        let style = Style::default()
            .fg(self.fg.unwrap_or(Color::Reset));

        canvas.render_widget(
            Paragraph::new(Span::styled(self.text.clone(), style)),
            self.size().rect_at_origin(),
        );
    }
}