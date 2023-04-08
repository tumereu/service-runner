use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;
use tui::widgets::{List as TuiList, Widget};

pub struct Measured<B: Backend> {
    pub size: Size,
    pub widget: Box<dyn Renderable<B>>
}

pub struct Size {
    pub width: u16,
    pub height: u16
}

pub trait Renderable<B: Backend> {
    fn render(self, rect: Rect, frame: &mut Frame<B>);
}

impl<W : Widget, B: Backend> Renderable<B> for W {
    fn render(self, rect: Rect, frame: &mut Frame<B>) {
        frame.render_widget(
            self,
            rect
        );
    }
}