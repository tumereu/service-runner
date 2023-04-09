use std::cmp::min;







#[derive(Clone, Copy)]
pub struct Size {
    pub width: u16,
    pub height: u16
}
impl Size {
    pub fn intersect(&self, other: Size) -> Size {
        (
            min(self.width, other.width),
            min(self.height, other.height)
        ).into()
    }
}

impl<X : Into<u16>, Y : Into<u16>> From<(X, Y)> for Size {
    fn from(value: (X, Y)) -> Self {
        Size {
            width: value.0.into(),
            height: value.1.into()
        }
    }
}
