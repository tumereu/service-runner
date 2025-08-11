use ratatui::layout::Rect;
use ratatui::prelude::Size;
use crate::space::Position;

pub trait RectAt {
    fn rect_at(&self, pos: Position) -> Rect;
}

pub trait RectAtOrigin {
    fn rect_at_origin(&self) -> Rect;
}

impl RectAt for Size {
    fn rect_at(&self, pos: Position) -> Rect {
        Rect {
            x: pos.x.try_into().unwrap_or_default(),
            y: pos.y.try_into().unwrap_or_default(),
            width: self.width,
            height: self.height,
        }
    }
}

impl<R : RectAt> RectAtOrigin for R {
    fn rect_at_origin(&self) -> Rect {
        self.rect_at(Position::origin())
    }
}