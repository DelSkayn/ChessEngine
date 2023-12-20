use std::env;

use chess_move_gen::{types::gen_type, InlineBuffer, MoveGenerator};
use common::board::Board;

struct Dropper<F: FnOnce()>(Option<F>);

impl<F: FnOnce()> Dropper<F> {
    fn new(f: F) -> Self {
        Dropper(Some(f))
    }

    fn forget(&mut self) {
        self.0.take();
    }
}

impl<F: FnOnce()> Drop for Dropper<F> {
    fn drop(&mut self) {
        if let Some(f) = self.0.take() {
            f()
        }
    }
}

fn main() {
    let mut args = env::args();
    args.next();
    let mut board = if let Some(x) = args.next() {
        let Ok(b) = x.parse() else {
            println!("failed to parse fen string");
            return;
        };
        b
    } else {
        Board::start_position()
    };

    let move_gen = MoveGenerator::new();
    for i in 1..=6 {
        let mut count = 0;
        perft(&move_gen, &mut board, i, &mut count, true);
        board.is_equal(&Board::start_position());
        println!("depth {}: {} nodes", i, count);
    }
}

fn perft(gen: &MoveGenerator, b: &mut Board, depth: usize, count: &mut usize, root: bool) {
    if depth == 0 {
        *count += 1;
        return;
    }
    let mut buffer = InlineBuffer::new();
    let position = gen.gen_info(b);
    if gen.check_mate(b, &position) {
        return;
    }
    gen.gen_moves_info::<gen_type::All>(b, &position, &mut buffer);

    for i in 0..buffer.len() {
        let m = buffer.get(i).unwrap();
        let last = *count;
        let m = b.make_move(m);
        {
            let mut dropper = Dropper::new(|| {
                println!("move : {}", m.mov);
            });
            perft(gen, b, depth - 1, count, false);
            //assert!(b.is_valid());
            dropper.forget()
        }
        if root {
            println!("nodes after '{}':{}", m.mov, *count - last);
        }
        b.unmake_move(m);
    }
}
