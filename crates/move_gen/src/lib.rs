mod generator;
pub mod inline_buffer;
mod tables;
pub mod types;

pub use generator::{MoveGenerator, PositionInfo};
pub use inline_buffer::InlineBuffer;
pub use tables::Tables;

#[cfg(test)]
mod test {
    use common::board::Board;

    use crate::types::gen_type;

    use super::*;

    fn perft(gen: &MoveGenerator, b: &mut Board, depth: usize, count: &mut usize, root: bool) {
        if depth == 0 {
            *count += 1;
            return;
        }
        let mut buffer = InlineBuffer::new();
        let position = gen.gen_info(b);
        if gen.check_mate(b, &position) {
            return;
        }
        gen.gen_moves_info::<gen_type::All>(b, &position, &mut buffer);

        for i in 0..buffer.len() {
            let m = buffer.get(i).unwrap();
            let last = *count;
            let m = b.make_move(m);
            perft(gen, b, depth - 1, count, false);
            if root {
                println!("nodes after '{}':{}", m.mov, *count - last);
            }
            b.unmake_move(m);
        }
    }

    #[test]
    fn perft_start_position() {
        let gen = MoveGenerator::new();
        let mut count = 0;

        perft(&gen, &mut Board::start_position(), 5, &mut count, true);
        assert_eq!(count, 4_865_609);
    }

    #[test]
    fn perft_position_1() {
        let gen = MoveGenerator::new();
        let mut count = 0;

        perft(
            &gen,
            &mut "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -"
                .parse()
                .unwrap(),
            5,
            &mut count,
            true,
        );
        assert_eq!(count, 193_690_690);
    }

    #[test]
    fn perft_position_2() {
        let gen = MoveGenerator::new();
        let mut count = 0;

        perft(
            &gen,
            &mut "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -".parse().unwrap(),
            6,
            &mut count,
            true,
        );
        assert_eq!(count, 11_030_083);
    }

    #[test]
    fn perft_position_3() {
        let gen = MoveGenerator::new();
        let mut count = 0;

        perft(
            &gen,
            &mut "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"
                .parse()
                .unwrap(),
            5,
            &mut count,
            true,
        );
        assert_eq!(count, 15_833_292);
    }

    #[test]
    fn perft_position_4() {
        let gen = MoveGenerator::new();
        let mut count = 0;

        perft(
            &gen,
            &mut "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8  "
                .parse()
                .unwrap(),
            5,
            &mut count,
            true,
        );
        assert_eq!(count, 89_941_194);
    }

    #[test]
    fn perft_position_5() {
        let gen = MoveGenerator::new();
        let mut count = 0;

        perft(
            &gen,
            &mut "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10 "
                .parse()
                .unwrap(),
            5,
            &mut count,
            true,
        );
        assert_eq!(count, 164_075_551);
    }
}
