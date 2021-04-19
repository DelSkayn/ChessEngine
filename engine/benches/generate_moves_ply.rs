use criterion::{criterion_group, criterion_main, Criterion};
use engine::{hash::Hasher, Board, Move, MoveGenerator};

pub fn gen_moves(
    gen: &MoveGenerator,
    hasher: &Hasher,
    b: &mut Board,
    buffers: &mut [Vec<Move>],
    depth: u32,
) {
    if depth == 0 {
        return;
    } else {
        let (buf, rest) = buffers.split_first_mut().unwrap();
        gen.gen_moves(&b, buf);
        for m in buf.iter().copied() {
            let undo = b.make_move(m, hasher);
            gen_moves(gen, hasher, b, rest, depth - 1);
            b.unmake_move(undo, hasher);
        }
        buf.clear();
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let hasher = Hasher::new();
    let mut board = Board::start_position();
    let move_gen = MoveGenerator::new();
    const V: Vec<Move> = Vec::new();
    let mut buffers = [V; 6];

    c.bench_function("generate_moves_ply", |b| {
        b.iter(|| {
            gen_moves(&move_gen, &hasher, &mut board, &mut buffers, 5);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
