use chess_core::{
    board2::{Board, EndChain},
    gen2::{gen_type, MoveGenerator},
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    let board = Board::start_position(EndChain);
    let move_gen = MoveGenerator::new();
    let mut buffer = Vec::new();

    c.bench_function("generate_moves_single", |b| {
        b.iter(|| {
            for _ in 0..100_000 {
                move_gen.gen_moves::<gen_type::All, _, _>(black_box(&board), &mut buffer);
                buffer.clear();
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
