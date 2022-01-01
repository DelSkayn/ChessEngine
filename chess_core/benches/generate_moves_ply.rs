use chess_core::{
    board2::{Board, EndChain},
    gen2::{gen_type, InlineBuffer, MoveGenerator},
};
use criterion::{criterion_group, criterion_main, Criterion};

pub fn gen_moves(gen: &MoveGenerator, b: &mut Board, depth: u32) {
    if depth == 0 {
        return;
    } else {
        let mut buf = InlineBuffer::<128>::new();
        gen.gen_moves::<gen_type::All, _, _>(&b, &mut buf);
        for m in buf.iter().copied() {
            let undo = b.make_move(m);
            gen_moves(gen, b, depth - 1);
            b.unmake_move(undo);
        }
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut board = Board::start_position(EndChain);
    let move_gen = MoveGenerator::new();

    c.bench_function("generate_moves_ply", |b| {
        b.iter(|| {
            gen_moves(&move_gen, &mut board, 5);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
