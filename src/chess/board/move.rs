use crate::chess::board::square::Square;

use bitflags::bitflags;
use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Flags(u8);

bitflags! {
    impl Flags: u8 {
        const PROMOTION = 0b1000;
        const CAPTURE = 0b0100;
        const SPECIAL1 = 0b0010;
        const SPECIAL2 = 0b0001;

        const QUIET = 0b0000;
        const PAWN_SPRINT = Self::SPECIAL2.bits();
        const CASTLE_KING = Self::SPECIAL1.bits();
        const CASTLE_QUEEN = Self::SPECIAL1.bits() | Self::SPECIAL2.bits();
        const EP_CAPTURE = Self::CAPTURE.bits() | Self::SPECIAL1.bits();
        const PROMOTION_KNIGHT = Self::PROMOTION.bits();
        const PROMOTION_BISHOP = Self::PROMOTION.bits() | Self::SPECIAL2.bits();
        const PROMOTION_ROOK = Self::PROMOTION.bits() | Self::SPECIAL1.bits();
        const PROMOTION_QUEEN = Self::PROMOTION.bits() | Self::SPECIAL1.bits() | Self::SPECIAL2.bits();
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Move {
    from: Square,
    to: Square,
    flags: Flags,
}

impl Move {
    pub fn new(from: Square, to: Square, flags: Flags) -> Self {
        Self { from, to, flags }
    }

    pub fn is_capture(&self) -> bool {
        self.flags.contains(Flags::CAPTURE)
    }

    pub fn is_en_passant(&self) -> bool {
        self.flags.contains(Flags::EP_CAPTURE)
    }

    pub fn is_promotion(&self) -> bool {
        self.flags.contains(Flags::PROMOTION)
    }

    pub fn is_pawn_sprint(&self) -> bool {
        self.flags == Flags::PAWN_SPRINT
    }

    pub fn from(&self) -> Square {
        self.from
    }

    pub fn to(&self) -> Square {
        self.to
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", self.from, self.to)
    }
}
