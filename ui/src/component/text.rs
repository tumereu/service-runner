use crate::component::{Component, MeasurableComponent};
use crate::frame_ctx::FrameContext;
use crate::UIResult;
use ratatui::layout::Size;
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use ratatui::widgets::Paragraph;

#[derive(Debug, Default)]
pub struct Text {
    pub text: String,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
}
impl Text {
    pub const ATTR_COLOR_FG: &'static str = "colors.Text.fg";

    pub fn new<S : Into<String>>(text: S) -> Self {
        Self {
            text: text.into(),
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
    type Output = ();

    fn render(self, context: &mut FrameContext) -> UIResult<Self::Output> {
        let mut style = Style::default()
            .fg(self.fg.unwrap_or(
                context.req_attr::<Color>(Self::ATTR_COLOR_FG)?.clone()
            ));

        if let Some(bg) = self.bg {
            style = style.bg(bg);
        }

        context.render_widget(
            Paragraph::new(Span::styled(self.text.clone(), style)),
            (0, 0).into(),
            context.size(),
        );

        Ok(())
    }
}
impl MeasurableComponent for Text {
    fn measure(&self, _context: &FrameContext) -> UIResult<Size> {
        Ok(self.size())
    }
}