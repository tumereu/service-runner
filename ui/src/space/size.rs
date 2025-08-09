#[derive(Debug, Clone)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl<W, H, E> TryFrom<(W, H)> for Size
where
    W: TryInto<u32, Error = E>,
    H: TryInto<u32, Error = E>,
{
    type Error = E;

    fn try_from((width, height): (W, H)) -> Result<Self, Self::Error> {
        Ok(Self {
            width: width.try_into()?,
            height: height.try_into()?,
        })
    }
}