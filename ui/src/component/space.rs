use crate::component::{Component, MeasurableComponent};
use crate::frame_ctx::FrameContext;
use crate::UIResult;
use ratatui::layout::Size;

#[derive(Debug, Default)]
pub struct Space {
    pub width: u16,
    pub height: u16,
}
impl Space {
    pub fn new<W : Into<u16>, H: Into<u16>>(width: W, height: H) -> Self {
        Self {
            width: width.into(),
            height: height.into(),
        }
    }
}

impl Component for Space {
    type Output = ();

    fn render(self, context: &mut FrameContext) -> UIResult<Self::Output> {
        Ok(())
    }
}
impl MeasurableComponent for Space {
    fn measure(&self, _context: &FrameContext) -> UIResult<Size> {
        Ok(Size { width: self.width, height: self.height })
    }
}