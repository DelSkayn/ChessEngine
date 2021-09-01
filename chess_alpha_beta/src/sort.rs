use super::AlphaBeta;
use crate::search::Line;
use chess_core::{gen3::MoveList, util::BoardArray, Move, Piece, Player};

pub struct MoveSorter<'a, M: MoveList> {
    moves: &'a mut M,
    hash_move: Option<Move>,
    cur: usize,
    pv: &'a mut Line,
    killer: &'a mut Line,
}

impl<'a, M: MoveList> MoveSorter<'a, M> {
    pub fn new(
        moves: &'a mut M,
        hash_move: Option<Move>,
        pv: &'a mut Line,
        killer: &'a mut Line,
    ) -> Self {
        Self {
            moves,
            hash_move,
            pv,
            killer,
        }
    }

    fn score_move(&self, m: Move) -> i32 {}
}
