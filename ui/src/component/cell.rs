use crate::component::{Component, MeasurableComponent};
use crate::space::RectAtOrigin;
use crate::{FrameContext, RenderArgs, UIError, UIResult};
use ratatui::layout::Size;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders};
use std::cmp::{max, min};

pub struct Cell<S : Default + 'static, O, C : MeasurableComponent<State = S, Output = O>> {
    content: Option<C>,
    pub bg: Option<Color>,
    pub border: Option<(Color, String)>,
    pub align_horiz: Align,
    pub align_vert: Align,
    pub padding_left: u16,
    pub padding_right: u16,
    pub padding_top: u16,
    pub padding_bottom: u16,
    pub min_width: u16,
    pub min_height: u16,
    pub max_width: u16,
    pub max_height: u16,
}
impl<S : Default + 'static, O, C : MeasurableComponent<State = S, Output = O>> Cell<S, O, C> {
    pub fn new(element: C) -> Cell<S, O, C> {
        Cell {
            content: Some(element),
            bg: None,
            border: None,
            align_horiz: Align::Stretch,
            align_vert: Align::Stretch,
            padding_left: 0,
            padding_right: 0,
            padding_top: 0,
            padding_bottom: 0,
            min_width: 0,
            min_height: 0,
            max_width: u16::MAX,
            max_height: u16::MAX,       
        }
    }

    pub fn bg(mut self, color: impl Into<Option<Color>>) -> Self {
        self.bg = color.into();
        self
    }

    pub fn border(mut self, color: Color, title: &str) -> Self {
        self.border = Some((color, title.into()));
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align_horiz = align;
        self.align_vert = align;
        self
    }

    pub fn align_horiz(mut self, align: Align) -> Self {
        self.align_horiz = align;
        self
    }

    pub fn align_vert(mut self, align: Align) -> Self {
        self.align_vert = align;
        self
    }

    pub fn padding(mut self, value: u16) -> Self {
        self.padding_left = value;
        self.padding_right = value;
        self.padding_top = value;
        self.padding_bottom = value;
        self
    }

    pub fn padding_left(mut self, left: u16) -> Self {
        self.padding_left = left;
        self
    }

    pub fn padding_right(mut self, right: u16) -> Self {
        self.padding_right = right;
        self
    }

    pub fn padding_top(mut self, top: u16) -> Self {
        self.padding_top = top;
        self
    }

    pub fn padding_bottom(mut self, bottom: u16) -> Self {
        self.padding_bottom = bottom;
        self
    }

    pub fn padding_vert(mut self, vert: u16) -> Self {
        self.padding_top = vert;
        self.padding_bottom = vert;
        self
    }

    pub fn padding_horiz(mut self, horiz: u16) -> Self {
        self.padding_left = horiz;
        self.padding_right = horiz;
        self
    }

    pub fn min_size(mut self, width: u16, height: u16) -> Self {
        self.min_width = width;
        self.min_height = height;
        self
    }

    pub fn min_width(mut self, width: u16) -> Self {
        self.min_width = width;
        self
    }

    pub fn min_height(mut self, height: u16) -> Self {
        self.min_height = height;
        self
    }
    
    pub fn max_width(mut self, width: u16) -> Self {
        self.max_width = width;
        self
    }
    
    pub fn max_height(mut self, height: u16) -> Self {
        self.max_height = height;
        self
    }
    
    pub fn width(mut self, width: u16) -> Self {
        self.min_width = width;
        self.max_width = width;
        self
    }
    
    pub fn height(mut self, height: u16) -> Self {
        self.min_height = height;
        self.max_height = height;
        self   
    }
}

impl<S : Default + 'static, O, C : MeasurableComponent<State = S, Output = O>> Component for Cell<S, O, C> {
    type State = ();
    type Output = O;

    fn render(&self, context: &FrameContext, _: &mut Self::State) -> UIResult<Self::Output> {
        let size = context.size();
        let size: Size = (
            min(size.width, self.max_width),
            min(size.height, self.max_height)
        ).into();
        
        if self.border.is_some() || self.bg.is_some() {
            let mut block = Block::default()
                .style(
                    Style::default()
                        .bg(self.bg.unwrap_or(Color::Reset))
                );
            if let Some((color, title)) = &self.border {
                block = block
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(*color))
                    .title(Line::from(title.to_string()).left_aligned());
            }
            context.render_widget(
                block,
                (0, 0).into(),
                size,
            );
        }

        let mut padding_left = self.padding_left;
        let mut padding_right = self.padding_right;
        let mut padding_top = self.padding_top;
        let mut padding_bottom = self.padding_bottom;

        if self.border.is_some() {
            padding_left += 1;
            padding_right += 1;
            padding_top += 1;
            padding_bottom += 1;
        }

        if let Some(content) = self.content.as_ref() {
            let content_size = context.measure_component::<S, C>("el", &content)?;
            let rect = size.rect_at_origin();

            let max_width = rect.width.saturating_sub(padding_left + padding_right);
            let max_height = rect.height.saturating_sub(padding_top + padding_bottom);

            let width = if self.align_horiz == Align::Stretch {
                max_width
            } else {
                min(content_size.width, max_width)
            };
            let height = if self.align_vert == Align::Stretch {
                max_height
            } else {
                min(content_size.height, max_height)
            };

            let x = match self.align_horiz {
                Align::Start | Align::Stretch => rect.x + padding_left,
                Align::End => rect.x + rect.width - width - padding_right,
                Align::Center => rect.x + (rect.width - width) / 2,
            };
            let y = match self.align_vert {
                Align::Start | Align::Stretch => rect.y + padding_top,
                Align::End => rect.y + rect.height - height - padding_bottom,
                Align::Center => rect.y + (rect.height - height) / 2,
            };

            context.render_component(
                RenderArgs::new(content)
                    .key("content")
                    .pos(x, y)
                    .size(width, height)
            )
        } else {
            Err(UIError::InvalidProp {
                msg: "Missing required property 'content'".to_string(),
            })
        }
    }
}

impl<S : Default + 'static, O, C : MeasurableComponent<State = S, Output = O>> MeasurableComponent for Cell<S, O, C> {
    fn measure(&self, context: &FrameContext, _: &Self::State) -> UIResult<Size> {
        let content_size = self
            .content
            .as_ref()
            .map(|content| context.measure_component::<S, C>("el", &content))
            .unwrap_or_else(|| Ok(Default::default()))?;

        let border_pad = if self.border.is_some() {
            2
        } else {
            0
        };

        let mut width = content_size.width + self.padding_left + self.padding_right + border_pad;
        width = max(width, self.min_width);
        width = min(width, self.max_width);

        let mut height = content_size.height + self.padding_top + self.padding_bottom + border_pad;
        height = max(height, self.min_height);
        height = min(height, self.max_height);       

        Ok((width, height).into())
    }
}

#[derive(Eq, PartialEq, Default, Debug, Clone, Copy)]
pub enum Align {
    #[default]
    Start,
    End,
    Center,
    Stretch,
}
