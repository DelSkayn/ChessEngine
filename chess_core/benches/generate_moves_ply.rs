use chess_core::{
    gen3::{gen_type, InlineBuffer, MoveGenerator},
    hash::Hasher,
    Board,
};
use criterion::{criterion_group, criterion_main, Criterion};

pub fn gen_moves(gen: &MoveGenerator, hasher: &Hasher, b: &mut Board, depth: u32) {
    if depth == 0 {
        return;
    } else {
        let mut buf = InlineBuffer::<128>::new();
        gen.gen_moves::<gen_type::All, _>(&b, &mut buf);
        for m in buf.iter().copied() {
            let undo = b.make_move(m, hasher);
            gen_moves(gen, hasher, b, depth - 1);
            b.unmake_move(undo, hasher);
        }
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let hasher = Hasher::new();
    let mut board = Board::start_position();
    let move_gen = MoveGenerator::new();

    c.bench_function("generate_moves_ply", |b| {
        b.iter(|| {
            gen_moves(&move_gen, &hasher, &mut board, 5);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
