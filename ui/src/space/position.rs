use std::ops::{Add, Sub};
use ratatui::layout::Offset;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32
}
impl Position {
    pub fn origin() -> Self {
        Self {
            x: 0,
            y: 0
        }
    }
}

impl Into<Offset> for Position {
    fn into(self) -> Offset {
        Offset {
            x: self.x,
            y: self.y
        }
    }
}

impl Into<Offset> for &Position {
    fn into(self) -> Offset {
        Offset {
            x: self.x,
            y: self.y
        }
    }
}

impl<W, H> Into<Position> for (W, H)
where
    W: Into<i32>,
    H: Into<i32> {

    fn into(self) -> Position {
        Position {
            x: self.0.into(),
            y: self.1.into(),
        }
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
