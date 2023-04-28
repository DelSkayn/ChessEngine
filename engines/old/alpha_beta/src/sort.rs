use crate::eval;

use super::Board;
use chess_core::{
    gen::{InlineBuffer, MoveList},
    Move,
};

pub struct MoveSorter<'a, const SIZE: usize> {
    moves: &'a mut InlineBuffer<SIZE>,
    hash_move: Option<Move>,
    pv_move: Option<Move>,
}

impl<'a, const SIZE: usize> MoveSorter<'a, SIZE> {
    const PIECE_VALUE: [i32; 12] = [
        0,
        eval::QUEEN_VALUE,
        eval::BISHOP_VALUE,
        eval::KNIGHT_VALUE,
        eval::ROOK_VALUE,
        eval::PAWN_VALUE,
        0,
        eval::QUEEN_VALUE,
        eval::BISHOP_VALUE,
        eval::KNIGHT_VALUE,
        eval::ROOK_VALUE,
        eval::PAWN_VALUE,
    ];

    pub fn new(
        moves: &'a mut InlineBuffer<SIZE>,
        hash_move: Option<Move>,
        pv_move: Option<Move>,
    ) -> Self {
        Self {
            moves,
            hash_move,
            pv_move,
        }
    }

    pub fn next_move(&mut self, board: &Board) -> Option<Move> {
        if self.moves.len() == 0 {
            return None;
        }

        let mut best = i32::MIN;
        let (idx, best_move) =
            self.moves
                .iter()
                .enumerate()
                .fold((0, Move::INVALID), |mut acc, m| {
                    let score = self.score_move(m.1, board);
                    if score > best {
                        best = score;
                        acc = m;
                    }
                    acc
                });
        self.moves.swap_remove(idx);
        return Some(best_move);
    }

    fn score_move(&self, m: Move, board: &Board) -> i32 {
        if Some(m) == self.pv_move {
            return 5000;
        }

        if Some(m) == self.hash_move {
            return 4000;
        }

        let from = board.on(m.from()).unwrap();

        if let Some(to) = board.on(m.to()) {
            Self::PIECE_VALUE[to as usize] - Self::PIECE_VALUE[from as usize] + eval::QUEEN_VALUE
        } else {
            0
        }
    }
}
