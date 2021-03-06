use crate::{
    gen3::{InlineBuffer, MoveList},
    Board, Move, Piece, Player,
};

use super::*;

impl Eval {
    pub fn alpha_beta_max(
        &mut self,
        b: &mut Board,
        mut alpha: i32,
        mut beta: i32,
        depth: u16,
        stop: &impl Fn() -> ShouldRun,
    ) -> i32 {
        if stop() == ShouldRun::Stop {
            return -Self::CHECK_VALUE;
        }

        let mut stored_best_move = None;
        if let Some(x) = self.hashmap.lookup(b.hash) {
            stored_best_move = Some(x.best_move);
            self.table_hits += 1;
            if x.depth >= depth {
                match x.value {
                    StoredValue::Exact(x) => return x,
                    StoredValue::LowerBound(x) => {
                        alpha = alpha.max(x);
                        if alpha >= beta {
                            self.cut_offs += 1;
                            return x;
                        }
                    }
                    StoredValue::UpperBound(x) => {
                        beta = beta.min(x);
                        if alpha >= beta {
                            self.cut_offs += 1;
                            return x;
                        }
                    }
                }
            }
        }

        if depth == 0 {
            return self.quiesce_max(b, alpha, beta);
        }

        let mut buffer = InlineBuffer::<128>::new();
        self.gen.gen_moves::<gen_type::All, _>(b, &mut buffer);
        if buffer.len() == 0 {
            let color = match b.state.player {
                crate::Player::White => -1,
                crate::Player::Black => 1,
            };
            return color * Self::CHECK_VALUE;
        }

        self.order_moves(b, &mut buffer, stored_best_move);

        let mut best_move = 0u16;

        for (idx, m) in buffer.iter().copied().enumerate() {
            let undo = b.make_move(m, &self.hasher);
            //assert!(b.is_valid(), "{:?}", b);
            let value = self.alpha_beta_min(b, alpha, beta, depth - 1, stop);
            b.unmake_move(undo, &self.hasher);
            if value > alpha {
                best_move = idx as u16;
                alpha = value;
            }
            if value >= beta {
                self.cut_offs += 1;
                break;
            }
        }

        if alpha >= beta {
            self.hashmap.store(Stored {
                hash: b.hash,
                depth,
                value: StoredValue::LowerBound(alpha),
                best_move,
            })
        } else {
            self.hashmap.store(Stored {
                hash: b.hash,
                depth,
                value: StoredValue::Exact(alpha),
                best_move,
            })
        }

        return alpha;
    }

    pub fn alpha_beta_min(
        &mut self,
        b: &mut Board,
        mut alpha: i32,
        mut beta: i32,
        depth: u16,
        stop: &impl Fn() -> ShouldRun,
    ) -> i32 {
        if stop() == ShouldRun::Stop {
            return Self::CHECK_VALUE;
        }

        let mut stored_best_move = None;
        if let Some(x) = self.hashmap.lookup(b.hash) {
            stored_best_move = Some(x.best_move);
            self.table_hits += 1;
            if x.depth >= depth {
                match x.value {
                    StoredValue::Exact(x) => return x,
                    StoredValue::LowerBound(x) => {
                        alpha = alpha.max(x);
                        if alpha >= beta {
                            self.cut_offs += 1;
                            return x;
                        }
                    }
                    StoredValue::UpperBound(x) => {
                        beta = beta.min(x);
                        if alpha >= beta {
                            self.cut_offs += 1;
                            return x;
                        }
                    }
                }
            }
        }

        if depth == 0 {
            return self.quiesce_min(b, alpha, beta);
        }

        let mut buffer = InlineBuffer::<128>::new();
        self.gen.gen_moves::<gen_type::All, _>(b, &mut buffer);
        if buffer.len() == 0 {
            return -Self::CHECK_VALUE;
        }

        self.order_moves(b, &mut buffer, stored_best_move);

        let mut best_move = 0;

        for (idx, m) in buffer.iter().copied().enumerate() {
            let undo = b.make_move(m, &self.hasher);
            //assert!(b.is_valid(), "{:?}", b);
            let value = self.alpha_beta_max(b, alpha, beta, depth - 1, stop);
            b.unmake_move(undo, &self.hasher);
            if value < beta {
                best_move = idx as u16;
                beta = value;
            }
            if beta <= alpha {
                self.cut_offs += 1;
                break;
            }
        }

        if alpha >= beta {
            self.hashmap.store(Stored {
                hash: b.hash,
                depth,
                value: StoredValue::UpperBound(beta),
                best_move,
            })
        } else {
            self.hashmap.store(Stored {
                hash: b.hash,
                depth,
                value: StoredValue::Exact(beta),
                best_move,
            })
        }

        return beta;
    }

    fn quiesce_max(&mut self, b: &mut Board, mut alpha: i32, beta: i32) -> i32 {
        let value = self.eval_board(b);
        if value >= beta {
            return beta;
        }
        if alpha < value {
            alpha = value
        }

        let mut buffer = InlineBuffer::<128>::new();
        self.gen.gen_moves::<gen_type::Captures, _>(b, &mut buffer);
        self.order_moves(b, &mut buffer, None);
        for m in buffer.iter().copied() {
            if b.on(m.to()).is_none() {
                continue;
            }
            let undo = b.make_move(m, &self.hasher);
            //assert!(b.is_valid(), "{:?}", b);
            let value = self.quiesce_min(b, alpha, beta);
            b.unmake_move(undo, &self.hasher);

            if value >= beta {
                return beta;
            }
            if value > alpha {
                alpha = value
            }
        }
        alpha
    }

    fn quiesce_min(&mut self, b: &mut Board, alpha: i32, mut beta: i32) -> i32 {
        let value = self.eval_board(b);
        if value <= alpha {
            return alpha;
        }
        if value < beta {
            beta = value
        }

        let mut buffer = InlineBuffer::<128>::new();
        self.gen.gen_moves::<gen_type::Captures, _>(b, &mut buffer);
        self.order_moves(b, &mut buffer, None);
        for m in buffer.iter().copied() {
            if b.on(m.to()).is_none() {
                continue;
            }
            let undo = b.make_move(m, &self.hasher);
            //assert!(b.is_valid(), "{:?}", b);
            let value = self.quiesce_max(b, alpha, beta);
            b.unmake_move(undo, &self.hasher);
            if value <= alpha {
                return alpha;
            }
            if value < beta {
                beta = value
            }
        }
        beta
    }

    pub fn eval_board(&mut self, b: &Board) -> i32 {
        self.nodes_evaluated += 1;

        if self.gen.check_mate(b) {
            let color = match b.state.player {
                Player::White => -1,
                Player::Black => 1,
            };
            return color * Self::CHECK_VALUE;
        }

        let mut piece_value = (b[Piece::WhiteQueen].count() as i32
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

        for p in b[Piece::WhiteKing].iter() {
            piece_value += Self::KING_TABLE[p.flip()]
        }
        for p in b[Piece::WhiteBishop].iter() {
            piece_value += Self::BISHOP_TABLE[p.flip()]
        }
        for p in b[Piece::WhiteKnight].iter() {
            piece_value += Self::KNIGHT_TABLE[p.flip()]
        }
        for p in b[Piece::WhitePawn].iter() {
            piece_value += Self::PAWN_TABLE[p.flip()]
        }
        for p in b[Piece::BlackKing].iter() {
            piece_value -= Self::KING_TABLE[p]
        }
        for p in b[Piece::BlackBishop].iter() {
            piece_value -= Self::BISHOP_TABLE[p]
        }
        for p in b[Piece::BlackKnight].iter() {
            piece_value -= Self::KNIGHT_TABLE[p]
        }
        for p in b[Piece::BlackPawn].iter() {
            piece_value -= Self::PAWN_TABLE[p]
        }

        piece_value
    }

    pub fn order_moves<T>(&mut self, b: &Board, moves: &mut T, stored_best_move: Option<u16>)
    where
        T: MoveList,
    {
        let move_swap = if let Some(x) = stored_best_move {
            if moves.len() > x as usize {
                moves.swap(0, x as usize);
                1
            } else {
                0
            }
        } else {
            0
        };

        if moves.len() == move_swap {
            return;
        }

        let mut best_value = self.eval_move(b, &moves.get(move_swap));
        for i in move_swap + 1..moves.len() {
            let v = self.eval_move(b, &moves.get(move_swap));
            if v > best_value {
                moves.swap(move_swap, i);
                best_value = v;
            }
        }
    }

    pub fn eval_move(&mut self, b: &Board, mov: &Move) -> i32 {
        let mut value = 0;
        if let Some(taken) = b.on(mov.to()) {
            value += self.value_lookup[taken] * 100 + self.value_lookup[b.on(mov.from()).unwrap()]
        }
        match mov.ty() {
            Move::TYPE_CASTLE => value += 8,
            Move::TYPE_PROMOTION => value += Self::PAWN_VALUE,
            _ => {}
        }
        value
    }
}
