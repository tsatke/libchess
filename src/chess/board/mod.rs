use std::ops::{Index, IndexMut, Range};

use arr_macro::arr;

use crate::chess::board::iter::SquareIterator;
use crate::chess::board::piece::{Color, Kind, Piece};
use crate::chess::board::r#move::{Flags, Move};
use crate::chess::board::square::Square::*;
use crate::chess::board::square::{Direction, Square};

pub mod r#move;
pub mod piece;
pub mod setup;
pub mod square;

mod iter;

const BOARD_SIZE: usize = 64;

pub enum MoveError {
    IllegalMove,
    KingInCheck,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Board {
    squares: [Option<Piece>; BOARD_SIZE],
    last_move: Option<Move>,
    castle_rights_white_kingside: bool,
    castle_rights_white_queenside: bool,
    castle_rights_black_kingside: bool,
    castle_rights_black_queenside: bool,
}

impl Board {
    pub fn new() -> Self {
        Self {
            squares: arr![None; 64], // need integer literal here
            last_move: None,
            castle_rights_white_kingside: true,
            castle_rights_white_queenside: true,
            castle_rights_black_kingside: true,
            castle_rights_black_queenside: true,
        }
    }

    pub fn squares(&self) -> impl Iterator<Item = Option<Piece>> + '_ {
        SquareIterator::new(self)
    }

    pub fn populate<S>(&mut self, setup: S)
    where
        S: Fn(&mut Board),
    {
        setup(self);
    }

    pub fn place(&mut self, square: Square, piece: Piece) {
        self[square] = Some(piece);
    }

    pub fn pieces_with_position(&self) -> Vec<(Square, Piece)> {
        self.squares
            .iter()
            .enumerate()
            .filter(|(_, x)| x.is_some())
            .map(|(i, x)| (Square::from(i), x.unwrap()))
            .collect()
    }

    pub fn pieces(&self) -> Vec<Piece> {
        #[allow(clippy::option_filter_map)] // with flatten we get Vec<&Piece>
        self.squares
            .iter()
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect()
    }

    /// Attempts to perform the given move on the current board. The board is not
    /// modified, but copied and returned as owned object.
    pub fn make_move(&self, mov: Move) -> Result<Board, MoveError> {
        let piece = match self[mov.from()] {
            None => return Err(MoveError::IllegalMove),
            Some(p) => p,
        };
        let color = piece.color();

        let mut b = self.clone();
        b[mov.from()] = None;
        b[mov.to()] = Some(piece);

        if b.king_in_check(color) {
            return Err(MoveError::KingInCheck);
        }

        b.last_move = Some(mov);
        Ok(b)
    }

    fn king_in_check(&mut self, color: Color) -> bool {
        // safe variant, slow AF

        // self.generate_moves(color.other())
        //     .into_iter()
        //     .find(|m| m.is_capture() && self[m.to()].map_or(false, |p| p.kind() == Kind::King))
        //     .is_some()

        // better variant, check more targeted

        let square_opt = self.find_king(color);
        if square_opt.is_none() {
            return false;
        }

        let square = square_opt.unwrap();
        // detect check by rook, bishop or queen
        for kind in [Kind::Rook, Kind::Bishop] {
            if self
                .generate_moves_sliding(color, square, kind)
                .into_iter()
                .map(|m| m.to())
                .find(|&s| {
                    if let Some(p) = self[s] {
                        return p.color() == color.other()
                            && match p.kind() {
                                Kind::Queen => true,
                                k => k == kind,
                            };
                    }
                    false
                })
                .is_some()
            {
                return true;
            }
        }
        // detect check by knight
        if self
            .generate_moves_knight(color, square)
            .into_iter()
            .map(|m| m.to())
            .find(|&s| {
                self[s].map_or(false, |p| {
                    p.kind() == Kind::Knight && p.color() == color.other()
                })
            })
            .is_some()
        {
            return true;
        }
        // detect check by pawn
        let (l, r) = match color {
            Color::Black => (Direction::DownLeft, Direction::DownRight),
            Color::White => (Direction::UpLeft, Direction::UpRight),
        };
        for dir in [l, r] {
            if Board::within_board_bounds(square, dir) {
                if let Some(piece) = self[square + dir] {
                    if piece.color() == color.other() && piece.kind() == Kind::Pawn {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn find_king(&self, color: Color) -> Option<Square> {
        for s in A1..=H8 {
            if let Some(p) = self[s] {
                if p.kind() == Kind::King && p.color() == color {
                    return Some(s);
                }
            }
        }
        None
    }

    pub fn generate_moves(&mut self, color: Color) -> Vec<Move> {
        self.pieces_with_position()
            .into_iter()
            .filter(|(_, x)| x.color() == color)
            .map(|(i, x)| match x.kind() {
                Kind::Pawn => self.generate_moves_pawn(color, i),
                Kind::King => self.generate_moves_king(color, i),
                Kind::Knight => self.generate_moves_knight(color, i),
                Kind::Bishop | Kind::Rook | Kind::Queen => {
                    self.generate_moves_sliding(color, i, x.kind())
                }
            })
            .flat_map(|x| x.into_iter())
            .collect()
    }

    fn generate_moves_pawn(&self, color: Color, square: Square) -> Vec<Move> {
        let mut result = Vec::new();

        if square.rank()
            == match color {
                Color::Black => 1,
                Color::White => 8,
            }
        {
            return result;
        }

        let (move_dir, home_row, promotion_possible_row) = match color {
            Color::Black => (Direction::Down, 7_u8, 2_u8),
            Color::White => (Direction::Up, 2_u8, 7_u8),
        };

        // normal moves
        if self[square + move_dir].is_none() {
            if square.rank() == promotion_possible_row {
                for promotion_flags in [
                    Flags::PROMOTION_BISHOP,
                    Flags::PROMOTION_KNIGHT,
                    Flags::PROMOTION_QUEEN,
                    Flags::PROMOTION_ROOK,
                ] {
                    result.push(Move::new(square, square + move_dir, promotion_flags));
                }
            } else {
                let mov = Move::new(square, square + move_dir, Flags::QUIET);
                result.push(mov);
            }

            // pawn sprint, but only if normal move is also possible
            if square.rank() == home_row && self[square + move_dir + move_dir].is_none() {
                let sprint = Move::new(square, square + move_dir + move_dir, Flags::PAWN_SPRINT);
                result.push(sprint);
            }
        }

        // captures
        let column = square.file();
        let can_capture = |target: Square| match self[target] {
            None => false,
            Some(piece) => piece.color() == color.other(),
        };
        let generate_promotion_capture_moves = |target: Square| {
            let mut capture_promotion_moves = Vec::new();

            if square.rank() == promotion_possible_row {
                for promotion_flags in [
                    Flags::PROMOTION_BISHOP,
                    Flags::PROMOTION_KNIGHT,
                    Flags::PROMOTION_QUEEN,
                    Flags::PROMOTION_ROOK,
                ] {
                    capture_promotion_moves.push(Move::new(
                        square,
                        target,
                        Flags::CAPTURE | promotion_flags,
                    ));
                }
            } else {
                let capture = Move::new(square, target, Flags::CAPTURE);
                capture_promotion_moves.push(capture);
            }

            capture_promotion_moves
        };
        // capture left
        if column != 1 && can_capture(square + Direction::UpLeft) {
            generate_promotion_capture_moves(square + Direction::UpLeft)
                .into_iter()
                .for_each(|m| result.push(m));
        }
        // capture right
        if column != 8 && can_capture(square + Direction::UpRight) {
            generate_promotion_capture_moves(square + Direction::UpRight)
                .into_iter()
                .for_each(|m| result.push(m));
        }

        // en passant
        let ep_possible = match &self.last_move {
            None => false,
            Some(mov) => mov.is_pawn_sprint(),
        };
        if ep_possible {
            let sprint_col = self.last_move.as_ref().unwrap().from().file();
            if column == sprint_col - 1 || column == sprint_col + 1 {
                let ep_base_row = match color {
                    Color::Black => 4,
                    Color::White => 5,
                };
                if square.rank() == ep_base_row {
                    let ep_move = Move::new(
                        square,
                        Square::from_coordinates(
                            match color {
                                Color::Black => 3,
                                Color::White => 6,
                            },
                            sprint_col,
                        ),
                        Flags::EP_CAPTURE,
                    );
                    result.push(ep_move);
                }
            }
        }

        result
    }

    fn generate_moves_king(&self, color: Color, square: Square) -> Vec<Move> {
        let mut result = Vec::new();

        // moves and captures
        for direction in [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
            Direction::UpLeft,
            Direction::UpRight,
            Direction::DownLeft,
            Direction::DownRight,
        ] {
            if !Board::within_board_bounds(square, direction) {
                continue;
            }
            if let Some(mov) = match self[square + direction] {
                None => Some(Move::new(square, square + direction, Flags::QUIET)),
                Some(p) if p.color() == color.other() => {
                    Some(Move::new(square, square + direction, Flags::CAPTURE))
                }
                Some(_) => None,
            } {
                result.push(mov);
            }
        }

        if color.king_square() == square && self.has_castle_rights(color) {
            // we can assume that the king is on its original square
            let base_row = square.rank();

            let is_kingside_rook_present = self[Square::from_coordinates(base_row, 8)]
                .map_or(false, |p| p.kind() == Kind::Rook && p.color() == color);
            let is_kingside_free = self[square + Direction::Right].is_none()
                && self[square + Direction::Right + Direction::Right].is_none();
            if is_kingside_rook_present && is_kingside_free {
                let kingside_castle = Move::new(
                    square,
                    square + Direction::Right + Direction::Right,
                    Flags::CASTLE_KING,
                );
                result.push(kingside_castle);
            }

            let is_queenside_rook_present = self[Square::from_coordinates(base_row, 1)]
                .map_or(false, |p| p.kind() == Kind::Rook && p.color() == color);
            let is_queenside_free = self[square + Direction::Left].is_none()
                && self[square + Direction::Left + Direction::Left].is_none();
            if is_queenside_rook_present && is_queenside_free {
                let queenside_castle = Move::new(
                    square,
                    square + Direction::Left + Direction::Left,
                    Flags::CASTLE_KING,
                );
                result.push(queenside_castle);
            }
        }

        result
    }

    fn generate_moves_sliding(&self, color: Color, square: Square, kind: Kind) -> Vec<Move> {
        let mut result = Vec::new();

        let directions = [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
            Direction::UpLeft,
            Direction::UpRight,
            Direction::DownLeft,
            Direction::DownRight,
        ];
        let r: Range<usize> = match kind {
            Kind::Bishop => (4..8),
            Kind::Rook => (0..4),
            Kind::Queen => (0..8),
            _ => panic!("{:?} @ {} is not a sliding piece", kind, square),
        };
        for &dir in &directions[r] {
            let mut current_square = square;
            while Board::within_board_bounds(current_square, dir) {
                let target_square = current_square + dir;
                match self[target_square] {
                    None => {
                        let mov = Move::new(square, target_square, Flags::QUIET);
                        result.push(mov);
                    }
                    Some(p) => {
                        if p.color() == color.other() {
                            let mov = Move::new(square, target_square, Flags::CAPTURE);
                            result.push(mov);
                        }
                        break;
                    }
                };

                current_square = target_square;
            }
        }

        result
    }

    fn generate_moves_knight(&self, color: Color, square: Square) -> Vec<Move> {
        let mut result = Vec::new();

        let directions = [
            Direction::UpUpLeft,
            Direction::UpUpRight,
            Direction::LeftLeftUp,
            Direction::LeftLeftDown,
            Direction::RightRightUp,
            Direction::RightRightDown,
            Direction::DownDownLeft,
            Direction::DownDownRight,
        ];

        for &dir in &directions {
            if Board::within_board_bounds(square, dir) {
                match self[square + dir] {
                    None => {
                        let mov = Move::new(square, square + dir, Flags::QUIET);
                        result.push(mov);
                    }
                    Some(p) => {
                        if p.color() == color.other() {
                            let mov = Move::new(square, square + dir, Flags::CAPTURE);
                            result.push(mov);
                        }
                    }
                };
            }
        }

        result
    }

    fn within_board_bounds(square: Square, direction: Direction) -> bool {
        match direction {
            Direction::Up => square.rank() < 8,
            Direction::Down => square.rank() > 1,
            Direction::Left => square.file() > 1,
            Direction::Right => square.file() < 8,
            Direction::UpLeft => square.file() > 1 && square.rank() < 8,
            Direction::UpRight => square.file() < 8 && square.rank() < 8,
            Direction::DownLeft => square.file() > 1 && square.rank() > 1,
            Direction::DownRight => square.file() < 8 && square.rank() > 1,
            // knight directions
            Direction::UpUpLeft => square.rank() < 7 && square.file() > 1,
            Direction::UpUpRight => square.rank() < 7 && square.file() < 8,
            Direction::LeftLeftUp => square.rank() < 8 && square.file() > 2,
            Direction::LeftLeftDown => square.rank() > 1 && square.file() > 2,
            Direction::RightRightUp => square.rank() < 8 && square.file() < 7,
            Direction::RightRightDown => square.rank() > 1 && square.file() < 7,
            Direction::DownDownLeft => square.rank() > 2 && square.file() > 1,
            Direction::DownDownRight => square.rank() > 2 && square.file() < 8,
        }
    }

    pub fn has_castle_rights(&self, color: Color) -> bool {
        match color {
            Color::Black => self.castle_rights_black_kingside || self.castle_rights_black_queenside,
            Color::White => self.castle_rights_white_kingside || self.castle_rights_white_queenside,
        }
    }
}

impl Index<Square> for Board {
    type Output = Option<Piece>;

    fn index(&self, index: Square) -> &Self::Output {
        &self.squares[index as usize]
    }
}

impl IndexMut<Square> for Board {
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        &mut self.squares[index as usize]
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chess::board::piece::{Color, Kind};
    use crate::chess::board::setup::default_setup;

    extern crate test;

    #[test]
    fn test_move_gen_default_setup() {
        let mut b = Board::new();
        b.populate(default_setup);

        assert_eq!(20, b.generate_moves(Color::White).len());
    }

    #[test]
    fn test_make_move_simple() {
        let mut b = Board::new();
        b.place(B2, Piece::new(Color::White, Kind::Pawn));
        assert_eq!(None, b.last_move);
        assert_eq!(Some(Piece::new(Color::White, Kind::Pawn)), b[B2]);
        assert_eq!(None, b[B4]);

        let test_move = Move::new(B2, B4, Flags::PAWN_SPRINT);
        let move_result = b.make_move(test_move.clone());
        assert!(move_result.is_ok());
        let new_board = move_result.ok().unwrap();
        let expected = Board {
            last_move: Some(test_move),
            squares: new_board.squares,
            ..b
        };
        assert_eq!(expected, new_board);
    }

    #[test]
    fn test_move_gen_knight_center_of_board() {
        for color in [Color::White, Color::Black] {
            let mut b = Board::new();
            b.place(C4, Piece::new(color, Kind::Knight));

            let moves = b.generate_moves(color);
            assert_eq!(8, moves.len());
            assert!(moves.contains(&Move::new(C4, A3, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, A5, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, B2, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, B6, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, D2, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, D6, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, E3, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, E5, Flags::QUIET)));
        }
    }

    #[test]
    fn test_move_gen_queen_capture() {
        for color in [Color::White, Color::Black] {
            let mut b = Board::new();
            b.place(A1, Piece::new(color, Kind::Queen));
            b.place(A2, Piece::new(color.other(), Kind::Queen));
            b.place(B2, Piece::new(color.other(), Kind::Queen));
            b.place(B1, Piece::new(color.other(), Kind::Queen));

            let moves = b.generate_moves(color);
            assert_eq!(3, moves.len());
            assert!(moves.contains(&Move::new(A1, A2, Flags::CAPTURE)));
            assert!(moves.contains(&Move::new(A1, B1, Flags::CAPTURE)));
            assert!(moves.contains(&Move::new(A1, B2, Flags::CAPTURE)));
        }
    }

    #[test]
    fn test_move_gen_queen_simple() {
        for color in [Color::White, Color::Black] {
            let mut b = Board::new();
            b.place(C4, Piece::new(color, Kind::Queen));

            let moves = b.generate_moves(color);
            assert_eq!(25, moves.len());
            assert!(moves.contains(&Move::new(C4, A2, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, B3, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, D3, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, E2, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, F1, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, B5, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, A6, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, D5, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, E6, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, F7, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, G8, Flags::QUIET)));

            assert!(moves.contains(&Move::new(C4, A4, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, B4, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, D4, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, E4, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, F4, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, G4, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, H4, Flags::QUIET)));

            assert!(moves.contains(&Move::new(C4, C1, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, C2, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, C3, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, C5, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, C6, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, C7, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, C8, Flags::QUIET)));
        }
    }

    #[test]
    fn test_move_gen_bishop_simple() {
        for color in [Color::White, Color::Black] {
            let mut b = Board::new();
            b.place(C4, Piece::new(color, Kind::Bishop));

            let moves = b.generate_moves(color);
            assert_eq!(11, moves.len());
            assert!(moves.contains(&Move::new(C4, A2, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, B3, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, D3, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, E2, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, F1, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, B5, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, A6, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, D5, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, E6, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, F7, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, G8, Flags::QUIET)));
        }
    }

    #[test]
    fn test_move_gen_rook_simple() {
        for color in [Color::White, Color::Black] {
            let mut b = Board::new();
            b.place(B2, Piece::new(color, Kind::Rook));

            let moves = b.generate_moves(color);
            assert_eq!(14, moves.len());
            assert!(moves.contains(&Move::new(B2, A2, Flags::QUIET)));
            assert!(moves.contains(&Move::new(B2, C2, Flags::QUIET)));
            assert!(moves.contains(&Move::new(B2, D2, Flags::QUIET)));
            assert!(moves.contains(&Move::new(B2, E2, Flags::QUIET)));
            assert!(moves.contains(&Move::new(B2, F2, Flags::QUIET)));
            assert!(moves.contains(&Move::new(B2, G2, Flags::QUIET)));
            assert!(moves.contains(&Move::new(B2, H2, Flags::QUIET)));

            assert!(moves.contains(&Move::new(B2, B1, Flags::QUIET)));
            assert!(moves.contains(&Move::new(B2, B3, Flags::QUIET)));
            assert!(moves.contains(&Move::new(B2, B4, Flags::QUIET)));
            assert!(moves.contains(&Move::new(B2, B5, Flags::QUIET)));
            assert!(moves.contains(&Move::new(B2, B6, Flags::QUIET)));
            assert!(moves.contains(&Move::new(B2, B7, Flags::QUIET)));
            assert!(moves.contains(&Move::new(B2, B8, Flags::QUIET)));
        }
    }

    #[test]
    fn test_move_gen_rook_14_moves() {
        // no matter where the rook is placed, if the rook is the only piece on the board,
        // it will always have 14 possible moves

        for square in A1..=H8 {
            for color in [Color::White, Color::Black] {
                let mut b = Board::new();
                b.place(square, Piece::new(color, Kind::Rook));
                assert_eq!(14, b.generate_moves(color).len());
            }
        }
    }

    #[test]
    fn test_move_gen_king_simple() {
        for color in [Color::White, Color::Black] {
            let mut b = Board::new();
            b.place(C4, Piece::new(color, Kind::King));

            let moves = b.generate_moves(color);
            assert_eq!(8, moves.len());
            assert!(moves.contains(&Move::new(C4, B3, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, B4, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, B5, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, C3, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, C5, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, D3, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, D4, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, D5, Flags::QUIET)));
        }
    }

    #[test]
    fn test_move_gen_king_capture_and_blocked() {
        for color in [Color::White, Color::Black] {
            let mut b = Board::new();
            b.place(C4, Piece::new(color, Kind::King));
            b.place(C3, Piece::new(color, Kind::Pawn));
            b.place(C5, Piece::new(color, Kind::Pawn));
            b.place(B4, Piece::new(color.other(), Kind::Pawn));
            b.place(D4, Piece::new(color.other(), Kind::Pawn));

            let moves = b.generate_moves_king(color, C4);
            assert_eq!(6, moves.len());
            assert!(moves.contains(&Move::new(C4, B3, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, B4, Flags::CAPTURE)));
            assert!(moves.contains(&Move::new(C4, B5, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, D3, Flags::QUIET)));
            assert!(moves.contains(&Move::new(C4, D4, Flags::CAPTURE)));
            assert!(moves.contains(&Move::new(C4, D5, Flags::QUIET)));
        }
    }

    #[test]
    fn test_move_gen_king_white_castle_kingside() {
        let mut b = Board::new();
        b.place(E1, Piece::new(Color::White, Kind::King));
        b.place(H1, Piece::new(Color::White, Kind::Rook));

        let moves = b.generate_moves_king(Color::White, E1);
        assert_eq!(6, moves.len());
        assert!(moves.contains(&Move::new(E1, D1, Flags::QUIET)));
        assert!(moves.contains(&Move::new(E1, D2, Flags::QUIET)));
        assert!(moves.contains(&Move::new(E1, E2, Flags::QUIET)));
        assert!(moves.contains(&Move::new(E1, F2, Flags::QUIET)));
        assert!(moves.contains(&Move::new(E1, F1, Flags::QUIET)));
        assert!(moves.contains(&Move::new(E1, G1, Flags::CASTLE_KING)));
    }

    #[test]
    fn test_move_gen_king_white_castle_kingside_no_castle_rights() {
        let mut b = Board::new();
        b.castle_rights_white_kingside = false;
        b.place(E1, Piece::new(Color::White, Kind::King));
        b.place(H1, Piece::new(Color::White, Kind::Rook));

        let moves = b.generate_moves_king(Color::White, E1);
        assert_eq!(6, moves.len());
        assert!(moves.contains(&Move::new(E1, D1, Flags::QUIET)));
        assert!(moves.contains(&Move::new(E1, D2, Flags::QUIET)));
        assert!(moves.contains(&Move::new(E1, E2, Flags::QUIET)));
        assert!(moves.contains(&Move::new(E1, F2, Flags::QUIET)));
        assert!(moves.contains(&Move::new(E1, F1, Flags::QUIET)));
    }

    #[test]
    fn test_move_gen_king_black_castle_kingside() {
        let mut b = Board::new();
        b.place(E8, Piece::new(Color::Black, Kind::King));
        b.place(H8, Piece::new(Color::Black, Kind::Rook));

        let moves = b.generate_moves_king(Color::Black, E8);
        assert_eq!(6, moves.len());
        assert!(moves.contains(&Move::new(E8, D8, Flags::QUIET)));
        assert!(moves.contains(&Move::new(E8, D7, Flags::QUIET)));
        assert!(moves.contains(&Move::new(E8, E7, Flags::QUIET)));
        assert!(moves.contains(&Move::new(E8, F7, Flags::QUIET)));
        assert!(moves.contains(&Move::new(E8, F8, Flags::QUIET)));
        assert!(moves.contains(&Move::new(E8, G8, Flags::CASTLE_KING)));
    }

    #[test]
    fn test_move_gen_king_black_castle_kingside_no_castle_rights() {
        let mut b = Board::new();
        b.castle_rights_white_kingside = false;
        b.place(E8, Piece::new(Color::Black, Kind::King));
        b.place(H8, Piece::new(Color::Black, Kind::Rook));

        let moves = b.generate_moves_king(Color::Black, E8);
        assert_eq!(6, moves.len());
        assert!(moves.contains(&Move::new(E8, D8, Flags::QUIET)));
        assert!(moves.contains(&Move::new(E8, D7, Flags::QUIET)));
        assert!(moves.contains(&Move::new(E8, E7, Flags::QUIET)));
        assert!(moves.contains(&Move::new(E8, F7, Flags::QUIET)));
        assert!(moves.contains(&Move::new(E8, F8, Flags::QUIET)));
    }

    #[test]
    fn test_move_gen_pawn_simple() {
        let mut b = Board::new();
        b.place(C2, Piece::new(Color::White, Kind::Pawn));

        let moves = b.generate_moves(Color::White);
        assert_eq!(2, moves.len());
        assert!(moves.contains(&Move::new(C2, C3, Flags::QUIET)));
        assert!(moves.contains(&Move::new(C2, C4, Flags::PAWN_SPRINT)));
    }

    #[test]
    fn test_move_gen_pawn_promotion() {
        let mut b = Board::new();
        b.place(C7, Piece::new(Color::White, Kind::Pawn));

        let moves = b.generate_moves(Color::White);
        assert_eq!(4, moves.len());
        assert!(moves.contains(&Move::new(C7, C8, Flags::PROMOTION_BISHOP)));
        assert!(moves.contains(&Move::new(C7, C8, Flags::PROMOTION_KNIGHT)));
        assert!(moves.contains(&Move::new(C7, C8, Flags::PROMOTION_QUEEN)));
        assert!(moves.contains(&Move::new(C7, C8, Flags::PROMOTION_ROOK)));
    }

    #[test]
    fn test_move_gen_pawn_promotion_capture() {
        let mut b = Board::new();
        b.place(C7, Piece::new(Color::White, Kind::Pawn));
        b.place(D8, Piece::new(Color::Black, Kind::Rook));

        let moves = b.generate_moves(Color::White);
        assert_eq!(8, moves.len());
        assert!(moves.contains(&Move::new(C7, C8, Flags::PROMOTION_BISHOP)));
        assert!(moves.contains(&Move::new(C7, C8, Flags::PROMOTION_KNIGHT)));
        assert!(moves.contains(&Move::new(C7, C8, Flags::PROMOTION_QUEEN)));
        assert!(moves.contains(&Move::new(C7, C8, Flags::PROMOTION_ROOK)));
        assert!(moves.contains(&Move::new(C7, D8, Flags::CAPTURE | Flags::PROMOTION_BISHOP)));
        assert!(moves.contains(&Move::new(C7, D8, Flags::CAPTURE | Flags::PROMOTION_KNIGHT)));
        assert!(moves.contains(&Move::new(C7, D8, Flags::CAPTURE | Flags::PROMOTION_QUEEN)));
        assert!(moves.contains(&Move::new(C7, D8, Flags::CAPTURE | Flags::PROMOTION_ROOK)));
    }

    #[test]
    fn test_en_passant() {
        let mut b = Board::new();
        b.place(C5, Piece::new(Color::White, Kind::Pawn));
        b.place(D5, Piece::new(Color::Black, Kind::Pawn));
        b.last_move = Some(Move::new(D7, D5, Flags::PAWN_SPRINT));

        let moves = b.generate_moves(Color::White);
        assert_eq!(2, moves.len());
        assert!(moves.contains(&Move::new(C5, C6, Flags::QUIET)));
        assert!(moves.contains(&Move::new(C5, D6, Flags::EP_CAPTURE)));
    }

    #[test]
    fn test_move_gen_pawn_blocked() {
        let mut b = Board::new();
        b.place(C2, Piece::new(Color::White, Kind::Pawn));
        b.place(C3, Piece::new(Color::Black, Kind::Pawn));

        let moves = b.generate_moves(Color::White);
        assert_eq!(0, moves.len());
    }

    #[test]
    fn test_move_gen_pawn_sprint_blocked_hostile() {
        let mut b = Board::new();
        b.place(C2, Piece::new(Color::White, Kind::Pawn));
        b.place(C4, Piece::new(Color::Black, Kind::Pawn));

        let moves = b.generate_moves(Color::White);
        assert_eq!(1, moves.len());
        assert!(moves.contains(&Move::new(C2, C3, Flags::QUIET)));
    }

    #[test]
    fn test_move_gen_pawn_sprint_blocked_friendly() {
        let mut b = Board::new();
        b.place(C2, Piece::new(Color::White, Kind::Pawn));
        b.place(C4, Piece::new(Color::White, Kind::Pawn));

        let moves = b.generate_moves(Color::White);
        assert_eq!(2, moves.len());
        assert!(moves.contains(&Move::new(C2, C3, Flags::QUIET)));
        assert!(moves.contains(&Move::new(C4, C5, Flags::QUIET)));
    }

    #[test]
    fn test_pieces_with_position() {
        let mut b = Board::new();
        b.place(A5, Piece::new(Color::White, Kind::Rook));

        let pieces = b.pieces_with_position();
        assert_eq!(1, pieces.len());
        assert!(pieces.contains(&(A5, Piece::new(Color::White, Kind::Rook))));
    }

    #[test]
    fn test_pieces_with_position_full_board() {
        let mut b = Board::new();
        for i in A1..=H8 {
            b.place(i, Piece::new(Color::Black, Kind::Pawn));
        }

        let pieces = b.pieces_with_position();
        assert_eq!(64, pieces.len());
        for i in A1..=H8 {
            assert!(pieces.contains(&(i, Piece::new(Color::Black, Kind::Pawn))));
        }
    }

    #[test]
    fn test_squares_iterator() {
        let mut b = Board::new();
        for i in A1..=H8 {
            b.place(i, Piece::new(Color::Black, Kind::Rook));
        }

        for sq in b.squares() {
            assert_eq!(Some(Piece::new(Color::Black, Kind::Rook)), sq);
        }
    }
}
