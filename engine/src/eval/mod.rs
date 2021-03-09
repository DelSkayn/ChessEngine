pub struct Eval {
    gen: MoveGenerator,
    moves_buffer: Vec<Vec<Move>>,
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
            moves_buffer: Vec::new(),
        }
    }

    pub fn eval(&mut self, b: &Board) -> i32 {}

    pub fn eval_step(
        &mut self,
        b: &Board,
        mut alpha: i32,
        mut beta: i32,
        depth: u32,
        max: bool,
    ) -> i32 {
        let moves = &mut self.moves_buffer[depth];
        moves.clear();
        self.gen.gen_moves(b, moves);
        if depth == 0 {
            if moves.size() == 0 {
                return if max { i32::MIN } else { i32::MAX };
            }
            return self.eval_board(b);
        }
        if max {
            let mut value = i32::MIN;
            for m in moves.iter().cloned() {
                let b = b.make_move(m);
                let eval = self.eval_step(&b, alpha, beta, depth - 1, !max).max(value);
                value = val.max(eval);
                alpha = alpha.max(eval);
                if alpha > beta {
                    break;
                }
            }
            return value;
        } else {
            let mut value = i32::MAX;
            for m in moves.iter().cloned() {
                let b = b.make_move(m);
                let eval = self.eval_step(&b, alpha, beta, depth - 1, !max).min(value);
                value = val.min(eval);
                beta = beta.min(eval);
                if alpha > beta {
                    break;
                }
            }
            return value;
        }
    }

    pub fn eval_board(&self, b: &Board) -> i32 {
        let piece_value = b[Piece::WhitePawn].count() as i32 * PAWN_VALUE
            - b[Piece::BlackPawn].count() as i32 * PAWN_VALUE
            + b[Piece::WhiteBishop].count() as i32 * BISHOP_VALUE
            - b[Piece::BlackBishop].count() as i32 * BISHOP_VALUE
            + b[Piece::WhiteKnight].count() as i32 * KNIGHT_VALUE
            - b[Piece::BlackKnight].count() as i32 * KNIGHT_VALUE
            + b[Piece::WhiteRook].count() as i32 * ROOK_VALUE
            - b[Piece::BlackRook].count() as i32 * ROOK_VALUE
            + b[Piece::WhiteQueen].count() as i32 * QUEEN_VALUE
            - b[Piece::BlackQueen].count() as i32 * QUEEN_VALUE;

        piece_value
    }
}
