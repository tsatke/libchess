use crate::chess::board::piece::{Color, Kind, Piece};
use crate::chess::board::*;

pub fn default_setup(b: &mut Board) {
    // white pieces
    b.place(A1, Piece::new(Color::White, Kind::Rook));
    b.place(B1, Piece::new(Color::White, Kind::Knight));
    b.place(C1, Piece::new(Color::White, Kind::Bishop));
    b.place(D1, Piece::new(Color::White, Kind::Queen));
    b.place(E1, Piece::new(Color::White, Kind::King));
    b.place(F1, Piece::new(Color::White, Kind::Bishop));
    b.place(G1, Piece::new(Color::White, Kind::Knight));
    b.place(H1, Piece::new(Color::White, Kind::Rook));
    for square in A2..=H2 {
        b.place(square, Piece::new(Color::White, Kind::Pawn));
    }

    // black pieces
    b.place(A8, Piece::new(Color::Black, Kind::Rook));
    b.place(B8, Piece::new(Color::Black, Kind::Knight));
    b.place(C8, Piece::new(Color::Black, Kind::Bishop));
    b.place(D8, Piece::new(Color::Black, Kind::Queen));
    b.place(E8, Piece::new(Color::Black, Kind::King));
    b.place(F8, Piece::new(Color::Black, Kind::Bishop));
    b.place(G8, Piece::new(Color::Black, Kind::Knight));
    b.place(H8, Piece::new(Color::Black, Kind::Rook));
    for square in A7..=H7 {
        b.place(square, Piece::new(Color::Black, Kind::Pawn));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_setup_works() {
        let mut b = Board::new();
        b.populate(default_setup);

        assert_eq!(Some(Piece::new(Color::White, Kind::Rook)), b[A1]);
        assert_eq!(Some(Piece::new(Color::White, Kind::Knight)), b[B1]);
        assert_eq!(Some(Piece::new(Color::White, Kind::Bishop)), b[C1]);
        assert_eq!(Some(Piece::new(Color::White, Kind::Queen)), b[D1]);
        assert_eq!(Some(Piece::new(Color::White, Kind::King)), b[E1]);
        assert_eq!(Some(Piece::new(Color::White, Kind::Bishop)), b[F1]);
        assert_eq!(Some(Piece::new(Color::White, Kind::Knight)), b[G1]);
        assert_eq!(Some(Piece::new(Color::White, Kind::Rook)), b[H1]);
        for square in A2..=H2 {
            assert_eq!(Some(Piece::new(Color::White, Kind::Pawn)), b[square]);
        }

        assert_eq!(Some(Piece::new(Color::Black, Kind::Rook)), b[A8]);
        assert_eq!(Some(Piece::new(Color::Black, Kind::Knight)), b[B8]);
        assert_eq!(Some(Piece::new(Color::Black, Kind::Bishop)), b[C8]);
        assert_eq!(Some(Piece::new(Color::Black, Kind::Queen)), b[D8]);
        assert_eq!(Some(Piece::new(Color::Black, Kind::King)), b[E8]);
        assert_eq!(Some(Piece::new(Color::Black, Kind::Bishop)), b[F8]);
        assert_eq!(Some(Piece::new(Color::Black, Kind::Knight)), b[G8]);
        assert_eq!(Some(Piece::new(Color::Black, Kind::Rook)), b[H8]);
        for square in A7..=H7 {
            assert_eq!(Some(Piece::new(Color::Black, Kind::Pawn)), b[square]);
        }
    }
}
