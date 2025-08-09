use ratatui::layout::Rect;
use crate::space::Position;

#[derive(Debug, Clone)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}
impl Size {
    pub fn rect_at(&self, pos: Position) -> Rect {
        Rect {
            x: saturating_i32_to_u16(pos.x),
            y: saturating_i32_to_u16(pos.y),
            width: self.width,
            height: self.height,       
        }
    }

    pub fn rect_at_origin(&self) -> Rect {
        self.rect_at(Position::origin())
    }
}

impl<W, H> Into<Size> for (W, H)
where
    W: Into<u16>,
    H: Into<u16> {

    fn into(self) -> Size {
        Size {
            width: self.0.into(),
            height: self.1.into(),
        }
    }
}

fn saturating_i32_to_u16(value: i32) -> u16 {
    if value < 0 {
        0
    } else if value > u16::MAX as i32 {
        u16::MAX
    } else {
        value as u16
    }
}
