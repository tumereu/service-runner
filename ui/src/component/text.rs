use ratatui::layout::Size;
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use ratatui::widgets::Paragraph;
use crate::frame_ctx::FrameContext;
use crate::component::{Component, MeasurableComponent};
use crate::UIResult;
use crate::space::RectAtOrigin;

#[derive(Debug, Default)]
pub struct Text {
    pub text: String,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
}
impl Text {
    pub const ATTR_DEFAULT_FG: &'static str = "theme.Text.fg";

    pub fn new(text: String) -> Self {
        Self {
            text,
            ..Default::default()
        }
    }

    pub fn fg(mut self, fg: impl Into<Option<Color>>) -> Self {
        self.fg = fg.into();
        self
    }

    pub fn bg(mut self, bg: impl Into<Option<Color>>) -> Self {
        self.bg = bg.into();
        self
    }

    pub fn size(&self) -> Size {
        (self.text.len() as u16, 1u16).into()
    }
}

impl Component for Text {
    type State = ();
    type Output = ();

    fn render(&self, context: &FrameContext, _state: &mut Self::State) -> UIResult<Self::Output> {
        let mut style = Style::default()
            .fg(self.fg.unwrap_or(
                context.req_attr::<Color>(Self::ATTR_DEFAULT_FG)?.clone()
            ));

        if let Some(bg) = self.bg {
            style = style.bg(bg);
        }

        context.render_widget(
            Paragraph::new(Span::styled(self.text.clone(), style)),
            context.size().rect_at_origin(),
        );

        Ok(())
    }
}
impl MeasurableComponent for Text {
    fn measure(&self, _context: &FrameContext, _state: &Self::State) -> UIResult<Size> {
        Ok(self.size())
    }
}