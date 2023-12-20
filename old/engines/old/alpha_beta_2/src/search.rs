use crate::{eval, sort::MoveSorter};

use super::{
    hash::{TableScore, TableValue},
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
pub const CHECKMATE_SCORE: i32 = 1_000_000;
const INIT_BOUND: i32 = 2_000_000;
const INVALID_SCORE: i32 = 2_121_212;
const MAX_DEPTH: u8 = 99;

impl<C: EngineControl> AlphaBeta<C> {
    pub fn should_stop(&self) -> bool {
        let nodes = self.nodes;
        self.control.should_stop()
            || self.limits.nodes.map(|x| x > nodes).unwrap_or(false)
            || self
                .time_limit
                .as_ref()
                .map(|x| x.should_stop(nodes))
                .unwrap_or(false)
    }

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

        let mut lower = INIT_BOUND;
        let mut upper = -INIT_BOUND;

        let mut hit_bound = false;

        'depth_loop: while self.depth <= MAX_DEPTH {
            let mut best_move = Move::INVALID;
            let mut line = Line::new();

            loop {
                let mut buffer = moves.clone();

                let pref_upper = upper;

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

                if self.should_stop() {
                    break 'depth_loop;
                }

                if !hit_bound && upper == lower || upper == pref_upper {
                    lower = INIT_BOUND;
                    upper = -INIT_BOUND;
                    hit_bound = true;
                    self.control.info(Info::Debug("RETRY".to_string()));
                } else {
                    break;
                }
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

            if self.should_stop()
                || self
                    .limits
                    .depth
                    .map(|x| x >= self.depth as u32)
                    .unwrap_or(false)
            {
                break;
            }

            if upper.abs() == CHECKMATE_SCORE {
                break;
            }

            lower = upper + eval::PAWN_VALUE / 4;
            upper = upper - eval::PAWN_VALUE / 4;
            hit_bound = false;

            self.depth += 1;
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
        if self.should_stop() {
            return -INVALID_SCORE;
        }

        let mut hash_move = None;
        if let Some(hash) = self.table.get(self.board.chain.hash) {
            if hash.depth >= depth {
                self.table_hit += 1;
                hash_move = Some(hash.r#move);
                match hash.score {
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
        }

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

        self.table.set(TableValue {
            hash: self.board.chain.hash,
            depth,
            r#move: best_move,
            score,
        });
        return value;
    }

    fn quiesce(&mut self, lower: i32, mut upper: i32, color: i32) -> i32 {
        let info = self.gen.gen_info(&self.board);
        let value = color * self.eval_board(&info);
        if value >= lower {
            return lower;
        }
        upper = upper.max(value);

        let mut buffer = InlineBuffer::<128>::new();
        self.gen
            .gen_moves_info::<gen_type::Captures, _, _>(&self.board, &info, &mut buffer);
        let mut sort = MoveSorter::new(&mut buffer, None, None);

        while let Some(m) = sort.next_move(&self.board) {
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
