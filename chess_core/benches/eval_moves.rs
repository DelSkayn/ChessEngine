use chess_core::{
    engine::{Engine, Info, ShouldRun},
    eval::Eval,
};
use criterion::{criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut eval = Eval::new();

    let mut cont = ShouldRun::Continue;

    c.bench_function("eval_moves", |b| {
        b.iter(|| {
            eval.go(
                &mut |i: Info| match i {
                    Info::Depth(d) => {
                        if d == 5 {
                            cont = ShouldRun::Stop
                        }
                        cont
                    }
                    _ => cont,
                },
                || ShouldRun::Continue,
            )
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
