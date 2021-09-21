use crate::chess::board::piece::Color;
use crate::chess::board::r#move::Move;
use crate::chess::board::Board;

pub fn perft(depth: usize, board: &mut Board, color: Color) -> usize {
    if depth == 0 {
        return 1;
    }
    board
        .generate_moves(color)
        .into_iter()
        .map(|m| board.make_move(m))
        .filter_map(|r| r.ok())
        .map(|mut b| perft(depth - 1, &mut b, color.other()))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chess::board::setup::default_setup;

    #[test]
    fn test_perft_table() {
        for table in [
            (0, 1),
            (1, 20),
            (2, 400),
            (3, 8902),
            (4, 197281),
            (5, 4865609),
        ] {
            let mut b = Board::new();
            b.populate(default_setup);

            assert_eq!(
                table.1,
                perft(table.0, &mut b, Color::White),
                "assert perft({}) == {}",
                table.0,
                table.1
            );
        }
    }
}
