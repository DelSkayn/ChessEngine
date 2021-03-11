use crate::{gen::MoveGenerator, Board, Move, Piece};

pub struct Eval {
    gen: MoveGenerator,
    nodes_evaluated: usize,
}

pub struct BestMove {
    pub mov: Option<Move>,
    pub value: i32,
    pub depth: usize,
    pub nodes_evaluated: usize,
}

#[derive(Default)]
pub struct Buffers {
    root: Vec<Move>,
    depth: Vec<Vec<Move>>,
}

impl Eval {
    pub const PAWN_VALUE: i32 = 300;
    pub const KNIGHT_VALUE: i32 = 300;
    pub const BISHOP_VALUE: i32 = 300;
    pub const ROOK_VALUE: i32 = 500;
    pub const QUEEN_VALUE: i32 = 800;

    pub fn new() -> Self {
        Eval {
            gen: MoveGenerator::new(),
            nodes_evaluated: 0,
        }
    }

    pub fn eval(
        &mut self,
        b: &Board,
        buffers: &mut Buffers,
        cont: &mut impl FnMut(Option<BestMove>) -> bool,
    ) {
        buffers.root.clear();
        self.gen.gen_moves(&b, &mut buffers.root);
        self.nodes_evaluated = 0;
        let mut depth = 1;
        let mut b = *b;
        loop {
            for _ in buffers.depth.len()..depth {
                buffers.depth.push(Vec::new());
            }

            let mut value = -i32::MAX;
            let mut best_move = None;
            for m in buffers.root.iter().copied() {
                let tmp = b;
                let undo = b.make_move(m);
                let new_value =
                    self.negamax(&mut b, &mut buffers.depth[0..depth], value, i32::MAX, 1);
                b.unmake_move(undo);
                assert!(tmp == b, "{:?}{:?} => move: {:?} {:?}", tmp, b, m, undo);
                if new_value > value {
                    best_move = Some(m);
                    value = new_value;
                }

                if !cont(None) {
                    return;
                }
            }

            let best_move = BestMove {
                mov: best_move,
                value,
                depth,
                nodes_evaluated: self.nodes_evaluated,
            };

            if !cont(Some(best_move)) {
                return;
            }

            depth += 1;
        }
    }

    fn negamax(
        &mut self,
        b: &mut Board,
        buffers: &mut [Vec<Move>],
        mut alpha: i32,
        beta: i32,
        color: i32,
    ) -> i32 {
        let (buffer, rest) = buffers.split_first_mut().unwrap();
        buffer.clear();
        self.gen.gen_moves(b, buffer);
        //println!("{:?}", buffer);
        if buffer.is_empty() {
            return -i32::MAX;
        }

        if rest.is_empty() {
            return color * self.eval_board(b);
        }

        let mut value = -i32::MAX;
        //dbg!(buffer.len());
        for m in buffer.iter().copied() {
            let tmp = *b;
            let undo = b.make_move(m);
            value = value.max(-self.negamax(b, rest, -beta, -alpha, -color));
            b.unmake_move(undo);
            assert!(tmp == *b, "{:?}{:?} => move: {:?} {:?}", tmp, b, m, undo);
            alpha = alpha.max(value);
            if alpha >= beta {
                break;
            }
        }

        return value;
    }

    pub fn eval_board(&mut self, b: &Board) -> i32 {
        self.nodes_evaluated += 1;

        let piece_value = (b[Piece::WhiteQueen].count() as i32
            - b[Piece::BlackQueen].count() as i32)
            * Self::QUEEN_VALUE
            + (b[Piece::WhiteRook].count() as i32 - b[Piece::BlackRook].count() as i32)
                * Self::ROOK_VALUE
            + (b[Piece::WhiteBishop].count() as i32 - b[Piece::BlackBishop].count() as i32)
                * Self::BISHOP_VALUE
            + (b[Piece::WhiteKnight].count() as i32 - b[Piece::BlackKnight].count() as i32)
                * Self::KNIGHT_VALUE
            + (b[Piece::WhitePawn].count() as i32 - b[Piece::BlackPawn].count() as i32)
                * Self::PAWN_VALUE;

        piece_value
    }
}
