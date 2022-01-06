use crate::{
    hash::{TableEntry, TableScore},
    sort::MoveSorter,
    AlphaBeta,
};

use chess_core::{
    engine::{EngineControl, Info},
    gen::{gen_type, InlineBuffer, MoveList},
    Move, Player,
};
use std::{mem::MaybeUninit, ptr};

#[derive(Debug)]
pub struct Line {
    v: [MaybeUninit<Move>; MAX_DEPTH as usize],
    len: usize,
}

impl Line {
    pub const fn new() -> Self {
        Line {
            v: [MaybeUninit::uninit(); MAX_DEPTH as usize],
            len: 0,
        }
    }

    #[inline]
    fn clear(&mut self) {
        self.len = 0;
    }

    #[inline]
    fn apply(&mut self, m: Move, other: &Line) {
        self.v[0] = MaybeUninit::new(m);
        unsafe {
            ptr::copy_nonoverlapping(&other.v[0], &mut self.v[1] as *mut _, other.len);
        }
        self.len = other.len + 1;
    }

    #[inline]
    pub fn get_pv(&mut self) -> &[Move] {
        unsafe { &*(&self.v[0..self.len] as *const [MaybeUninit<Move>] as *const [Move]) }
    }

    #[inline]
    pub fn get(&self, idx: u8) -> Option<Move> {
        if (idx as usize) < self.len {
            unsafe { Some(self.v[idx as usize].assume_init()) }
        } else {
            None
        }
    }
}

pub const INIT_BOUND: i32 = 32_001;
pub const INVALID_SCORE: i32 = 32_002;
pub const MAX_DEPTH: u8 = 99;
pub const CHECKMATE_SCORE: i32 = 32_000;

impl<C: EngineControl> AlphaBeta<C> {
    pub fn go_search(&mut self) -> Option<Move> {
        self.nodes = 0;
        self.table_hit = 0;

        let mut moves = InlineBuffer::<256>::new();
        self.gen
            .gen_moves::<gen_type::All, _, _>(&self.board, &mut moves);

        if moves.len() == 0 {
            return None;
        }

        let color = match self.board.state.player {
            Player::White => 1,
            Player::Black => -1,
        };

        self.pv.clear();

        self.depth = 1;

        let mut best_move_total = Move::INVALID;

        while self.depth <= MAX_DEPTH {
            let lower = INIT_BOUND;
            let mut upper = -INIT_BOUND;
            let mut line = Line::new();
            let mut best_move = Move::INVALID;
            let mut buffer = moves.clone();

            let mut sort = MoveSorter::new(&mut buffer, None, self.pv.get(0));

            while let Some(m) = sort.next_move(&self.board) {
                let undo = self.board.make_move(m);
                let value = -self.search(self.depth - 1, -upper, -lower, -color, &mut line);
                self.board.unmake_move(undo);
                if value > upper {
                    self.pv.apply(m, &line);
                    upper = value;
                    best_move = m;
                }
            }

            if self.control.should_stop() {
                break;
            }

            best_move_total = best_move;

            self.control.info(Info::Depth(self.depth as u16));
            self.control.info(Info::BestMove {
                mov: best_move_total,
                value: color * upper,
            });
            self.control.info(Info::Nodes(self.nodes as usize));
            self.control.info(Info::TransHit(self.table_hit as usize));
            self.control.info(Info::Pv(self.pv.get_pv().to_vec()));
            self.control.info(Info::Round);

            if self.control.should_stop() {
                break;
            }

            if upper.abs() == CHECKMATE_SCORE {
                break;
            }

            self.depth += 1;

            self.table.increment_generation();
        }

        if best_move_total != Move::INVALID {
            Some(best_move_total)
        } else {
            None
        }
    }

    fn search(
        &mut self,
        depth: u8,
        mut lower: i32,
        mut upper: i32,
        color: i32,
        pv_line: &mut Line,
    ) -> i32 {
        if self.control.should_stop() {
            return -INVALID_SCORE;
        }

        let mut hash_move = None;
        let hash_entry = match self.table.get(self.board.chain.hash) {
            TableEntry::Miss(x) => x,
            TableEntry::Hit(hit) => {
                hash_move = Some(hit.r#move());
                if hit.depth() >= depth {
                    self.table_hit += 1;
                    match hit.score() {
                        TableScore::Exact(x) => return x,
                        TableScore::Upper(x) => {
                            upper = upper.max(x);
                            if upper >= lower {
                                return x;
                            }
                        }
                        TableScore::Lower(x) => {
                            lower = lower.max(x);
                            if upper >= lower {
                                return x;
                            }
                        }
                    }
                }
                hit.into_entry()
            }
        };

        if depth == 0 {
            let q = self.quiesce(lower, upper, color);
            assert_ne!(q.abs(), INIT_BOUND);
            return q;
        }

        let mut buffer = InlineBuffer::<128>::new();
        let pos_info = self
            .gen
            .gen_moves::<gen_type::All, _, _>(&self.board, &mut buffer);

        if self.gen.drawn(&self.board, &pos_info) {
            return -self.contempt;
        }

        if buffer.len() == 0 {
            if self.gen.checked_king(&self.board, &pos_info) {
                return -CHECKMATE_SCORE;
            } else {
                return -self.contempt;
            }
        }

        let mut value = -INIT_BOUND;

        let mut new_line = Line::new();

        let pv_move = self.pv.get(self.depth - depth);
        let mut sort = MoveSorter::new(&mut buffer, hash_move, pv_move);

        let mut best_move = Move::INVALID;

        while let Some(m) = sort.next_move(&self.board) {
            let undo = self.board.make_move(m);
            value = value.max(-self.search(depth - 1, -upper, -lower, -color, &mut new_line));
            self.board.unmake_move(undo);
            if value > upper {
                best_move = m;
                upper = value;
                pv_line.apply(m, &new_line);
            }
            if upper >= lower {
                break;
            }
        }

        let score = if value <= upper {
            TableScore::Lower(value)
        } else if value >= lower {
            TableScore::Upper(value)
        } else {
            TableScore::Exact(value)
        };

        self.table
            .write(hash_entry, self.board.chain.hash, depth, score, best_move);
        return value;
    }

    fn quiesce(&mut self, lower: i32, mut upper: i32, color: i32) -> i32 {
        let pos_info = self.gen.gen_info(&self.board);
        let value = color * self.eval_board(&pos_info);
        if value >= lower {
            return lower;
        }
        upper = upper.max(value);

        let mut buffer = InlineBuffer::<128>::new();
        self.gen
            .gen_moves_info::<gen_type::Captures, _, _>(&self.board, &pos_info, &mut buffer);

        for m in buffer.iter() {
            let undo = self.board.make_move(m);
            let value = -self.quiesce(-upper, -lower, -color);
            self.board.unmake_move(undo);

            if value >= lower {
                return lower;
            }
            upper = upper.max(value);
        }
        upper
    }
}
