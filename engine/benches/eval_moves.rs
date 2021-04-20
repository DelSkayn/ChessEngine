use criterion::{criterion_group, criterion_main, Criterion};
use engine::{
    eval::{BestMove, Buffers, Eval},
    hash::Hasher,
    Board,
};

pub fn criterion_benchmark(c: &mut Criterion) {
    let board = Board::start_position();
    let hasher = Hasher::new();
    let mut eval = Eval::new(hasher, 1 << 16);

    c.bench_function("eval_moves", |b| {
        b.iter(|| {
            eval.eval(&board, &mut |m: Option<BestMove>| {
                if let Some(m) = m {
                    !(m.depth == 4 && m.mov.is_some())
                } else {
                    true
                }
            })
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
