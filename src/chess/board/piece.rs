use crate::chess::board::Square;
use crate::chess::board::Square::{E1, E8};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Piece {
    color: Color,
    kind: Kind,
}

impl Piece {
    pub fn new(color: Color, kind: Kind) -> Self {
        Self { color, kind }
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Kind {
    Bishop,
    King,
    Knight,
    Pawn,
    Queen,
    Rook,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Color {
    Black,
    White,
}

impl Color {
    pub const fn other(&self) -> Color {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }

    pub const fn king_square(&self) -> Square {
        match self {
            Color::Black => E8,
            Color::White => E1,
        }
    }
}
