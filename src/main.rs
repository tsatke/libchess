use libchess::chess::board::piece::Color;
use libchess::chess::board::setup::default_setup;
use libchess::chess::board::Board;
use libchess::chess::perft::perft;
use std::time::Instant;

fn main() {
    let mut b = Board::new();
    b.populate(default_setup);

    let start = Instant::now();
    perft(5, &mut b, Color::White);
    let end = Instant::now();
    println!("Time elapsed: {:?}", end - start);
}
