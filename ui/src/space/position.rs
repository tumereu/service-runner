use std::ops::{Add, Sub};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Position {
    x: i32,
    y: i32
}

impl<W, H, E> TryFrom<(W, H)> for Position
where
    W: TryInto<i32, Error = E>,
    H: TryInto<i32, Error = E>,
{
    type Error = E;

    fn try_from((x, y): (W, H)) -> Result<Self, Self::Error> {
        Ok(Self {
            x: x.try_into()?,
            y: y.try_into()?,
        })
    }
}

impl Add for Position {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
impl Add<&Position> for Position {
    type Output = Self;

    fn add(self, other: &Position) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
impl Add<Position> for &Position {
    type Output = Position;

    fn add(self, other: Position) -> Position {
        Position {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
impl Add<&Position> for &Position {
    type Output = Position;

    fn add(self, other: &Position) -> Position {
        Position {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Position {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
impl Sub<&Position> for Position {
    type Output = Self;

    fn sub(self, other: &Position) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
impl Sub<Position> for &Position {
    type Output = Position;

    fn sub(self, other: Position) -> Position {
        Position {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
impl Sub<&Position> for &Position {
    type Output = Position;

    fn sub(self, other: &Position) -> Position {
        Position {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
