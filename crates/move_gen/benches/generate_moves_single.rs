use chess_move_gen::{types::gen_type, InlineBuffer, MoveGenerator};
use common::board::Board;
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

pub fn criterion_benchmark(c: &mut Criterion) {
    let board = Board::start_position();
    let move_gen = MoveGenerator::new();
    let mut buffer = InlineBuffer::new();

    c.bench_function("generate_moves_single", |b| {
        b.iter(|| {
            move_gen.gen_moves::<gen_type::All>(black_box(&board), &mut buffer);
            buffer.clear();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
