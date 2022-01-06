use chess_core::{
    engine::{Engine, EngineLimit, NoControl},
    eval::Eval,
};
use criterion::{criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut eval = Eval::new();

    c.bench_function("eval_moves", |b| {
        b.iter(|| eval.go(NoControl, None, EngineLimit::depth(5)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
