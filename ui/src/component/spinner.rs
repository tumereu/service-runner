use std::time::Instant;
use crate::component::{Component, MeasurableComponent, Space, Text};
use crate::frame_ctx::FrameContext;
use crate::UIResult;
use ratatui::layout::{Rect, Size};
use once_cell::sync::Lazy;
use ratatui::prelude::{Color, Span, Style};
use ratatui::widgets::Paragraph;

static REFERENCE_INSTANT: Lazy<Instant> = Lazy::new(Instant::now);
const SPINNER_CHARS: &[&str] = &["⠋", "⠙", "⠸", "⠴", "⠦", "⠇"];

#[derive(Debug, Default)]
pub struct Spinner {
    active: bool,
    fg: Option<Color>
}
impl Spinner {
    pub fn new(active: bool) -> Self {
        Self {
            active,
            fg: None
        }
    }
}

impl Component for Spinner {
    type Output = ();

    fn render(&self, context: &mut FrameContext) -> UIResult<Self::Output> {
        let style = Style::default()
            .fg(self.fg.unwrap_or(
                context.req_attr::<Color>(Text::ATTR_COLOR_FG)?.clone()
            ));

        let icon = if !self.active {
            " "
        } else {
            let phase: u128 = (Instant::now()
                .duration_since(*REFERENCE_INSTANT)
                .as_millis()
                / 100)
                % (SPINNER_CHARS.len() as u128);
            SPINNER_CHARS[phase as usize]
        };

        context.render_widget(
            Paragraph::new(Span::styled(icon, style)),
            (0, 0).into(),
            (1, 1).into()
        );
        
        Ok(())
    }
}
impl MeasurableComponent for Spinner {
    fn measure(&self, _context: &FrameContext) -> UIResult<Size> {
        Ok(Size { width: 1, height: 1 })
    }
}