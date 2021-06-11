use chess_core::{
    gen3::{gen_type, InlineBuffer, MoveGenerator, MoveList},
    hash::Hasher,
    Board,
};
use std::env;

fn main() {
    let mut args = env::args();
    args.next();
    let mut board = if let Some(x) = args.next() {
        Board::from_fen(&x).unwrap()
    } else {
        Board::start_position()
    };

    let hasher = Hasher::new();
    let move_gen = MoveGenerator::new();
    for i in 1..=6 {
        let mut count = 0;
        perft(&move_gen, &mut board, &hasher, i, &mut count, true);
        println!("depth {}: {} nodes", i, count);
    }
}

fn perft(
    gen: &MoveGenerator,
    b: &mut Board,
    hasher: &Hasher,
    depth: usize,
    count: &mut usize,
    root: bool,
) {
    if depth == 0 {
        *count += 1;
        return;
    }
    let mut buffer = InlineBuffer::<128>::new();
    gen.gen_moves::<gen_type::All, _>(b, &mut buffer);
    for i in 0..buffer.len() {
        let m = buffer.get(i);
        let last = *count;
        let m = b.make_move(m, hasher);
        perft(gen, b, hasher, depth - 1, count, false);
        if root {
            println!("nodes after '{}':{}", m.mov, *count - last);
        }
        b.unmake_move(m, hasher);
    }
}
