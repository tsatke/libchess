use crate::chess::board::piece::Piece;
use crate::chess::board::square::Square;
use crate::chess::board::square::Square::{A1, H8};
use crate::chess::board::Board;
use std::ops::RangeInclusive;

pub struct SquareIterator<'a> {
    board: &'a Board,
    range: RangeInclusive<Square>,
}

impl<'a> SquareIterator<'a> {
    pub fn new(board: &'a Board) -> Self {
        Self {
            board,
            range: A1..=H8,
        }
    }
}

impl Iterator for SquareIterator<'_> {
    type Item = Option<Piece>;

    fn next(&mut self) -> Option<Self::Item> {
        self.range.next().map(|s| self.board[s])
    }
}
