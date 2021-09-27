use crate::chess::board::piece::Color;
use crate::chess::board::Board;

pub fn perft(depth: usize, board: &mut Board, color: Color) -> usize {
    if depth == 0 {
        return 1;
    }

    let mut result: usize = 0;

    let moves = board.generate_moves(color);
    for mov in moves {
        let modification = board.make_move(mov.clone());
        if !board.king_in_check(color) {
            result += perft(depth - 1, board, color.other());
        }

        board.unmake_move(modification);
    }

    result

    // board
    //     .generate_moves(color)
    //     .into_iter()
    //     .map(|m| board.make_move(m))
    //     .filter_map(|r| r.ok())
    //     .map(|mut b| perft(depth - 1, &mut b, color.other()))m
    //     .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chess::board::setup::default_setup;

    #[test]
    fn performance_perft_5() {
        // 2.07 seconds before modification
        // 1.74 seconds after modification
        // cargo test --color=always --package libchess --lib chess::perft::tests::performance_perft_5 --release -- --exact

        let mut b = Board::new();
        b.populate(default_setup);
        perft(5, &mut b, Color::White);
    }

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
