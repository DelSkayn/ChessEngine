use criterion::{criterion_group, criterion_main, Criterion};
use engine::{Board, Move, MoveGenerator};

pub fn gen_moves(gen: &MoveGenerator, b: Board, buffers: &mut [Vec<Move>], depth: u32) {
    if depth == 0 {
        return;
    } else {
        let (buf, rest) = buffers.split_first_mut().unwrap();
        gen.gen_moves(&b, buf);
        for m in buf.iter().copied() {
            let new_board = b.make_move(m).flip();
            gen_moves(gen, new_board, rest, depth - 1);
        }
        buf.clear();
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let board =
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    let move_gen = MoveGenerator::new();
    const V: Vec<Move> = Vec::new();
    let mut buffers = [V; 6];

    c.bench_function("generate_moves_ply", |b| {
        b.iter(|| {
            gen_moves(&move_gen, board, &mut buffers, 5);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
