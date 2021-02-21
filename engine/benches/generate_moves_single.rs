use criterion::{black_box, criterion_group, criterion_main, Criterion};
use engine::{Board, Move, MoveGenerator};

pub fn criterion_benchmark(c: &mut Criterion) {
    let board =
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    let move_gen = MoveGenerator::new();
    let mut buffer = Vec::new();

    c.bench_function("generate_moves_single", |b| {
        b.iter(|| {
            for _ in 0..100_000 {
                move_gen.gen_moves(black_box(&board), &mut buffer);
                buffer.clear();
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
