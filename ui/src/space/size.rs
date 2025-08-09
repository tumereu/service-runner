#[derive(Debug, Clone)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl<W, H> Into<Size> for (W, H)
where
    W: Into<u32>,
    H: Into<u32> {

    fn into(self) -> Size {
        Size {
            width: self.0.into(),
            height: self.1.into(),
        }
    }
}