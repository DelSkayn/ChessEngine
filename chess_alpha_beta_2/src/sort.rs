use super::{AlphaBeta, Board};
use chess_core::{
    gen::{InlineBuffer, MoveList},
    Move,
};

pub struct MoveSorter<'a, const SIZE: usize> {
    moves: &'a mut InlineBuffer<SIZE>,
    hash_move: Option<Move>,
    pv_move: Option<Move>,
    sort_count: u8,
}

impl<'a, const SIZE: usize> MoveSorter<'a, SIZE> {
    const LIMIT_SORT: u8 = 20;

    const PIECE_VALUE: [i32; 12] = [
        0,
        AlphaBeta::QUEEN_VALUE,
        AlphaBeta::BISHOP_VALUE,
        AlphaBeta::KNIGHT_VALUE,
        AlphaBeta::ROOK_VALUE,
        AlphaBeta::PAWN_VALUE,
        0,
        AlphaBeta::QUEEN_VALUE,
        AlphaBeta::BISHOP_VALUE,
        AlphaBeta::KNIGHT_VALUE,
        AlphaBeta::ROOK_VALUE,
        AlphaBeta::PAWN_VALUE,
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
            sort_count: 0,
        }
    }

    pub fn next_move(&mut self, board: &Board) -> Option<Move> {
        if self.moves.len() == 0 {
            return None;
        }

        if self.sort_count < Self::LIMIT_SORT {
            let mut best = self.score_move(self.moves.get(0), board);
            let mut sorted = true;
            for i in 1..self.moves.len() {
                let score = self.score_move(self.moves.get(i), board);
                if score > best {
                    best = score;
                } else {
                    sorted = false;
                    self.moves.swap(i - 1, i);
                }
            }

            self.sort_count += if sorted { Self::LIMIT_SORT } else { 1 };
        }

        return self.moves.pop();
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
            Self::PIECE_VALUE[to as usize] - Self::PIECE_VALUE[from as usize]
                + AlphaBeta::QUEEN_VALUE
        } else {
            0
        }
    }
}
