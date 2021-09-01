use super::{
    hash::{TableScore, TableValue},
    AlphaBeta,
};
use chess_core::{
    engine::{Info, ShouldRun},
    gen3::{gen_type, InlineBuffer, MoveList},
    Move, Player,
};
use std::{mem::MaybeUninit, ptr};

#[derive(Debug)]
pub struct Line {
    v: [MaybeUninit<Move>; AlphaBeta::MAX_DEPTH as usize],
    len: usize,
}

impl Line {
    pub const fn new() -> Self {
        Line {
            v: [MaybeUninit::uninit(); AlphaBeta::MAX_DEPTH as usize],
            len: 0,
        }
    }

    fn clear(&mut self) {
        self.len = 0;
    }

    fn apply(&mut self, m: Move, other: &Line) {
        self.v[0] = MaybeUninit::new(m);
        unsafe {
            ptr::copy_nonoverlapping(&other.v[0], &mut self.v[1] as *mut _, other.len);
        }
        self.len = other.len + 1;
    }

    pub fn get_pv(&mut self) -> &[Move] {
        unsafe { &*(&self.v[0..self.len] as *const [MaybeUninit<Move>] as *const [Move]) }
    }
}

impl AlphaBeta {
    const CHECKMATE_SCORE: i32 = i32::MAX - 10;
    const MAX_DEPTH: u8 = 8;

    pub fn go_search<F: FnMut(Info) -> ShouldRun, Fc: Fn() -> ShouldRun>(
        &mut self,
        mut f: F,
        fc: Fc,
    ) -> Option<Move> {
        self.nodes = 0;

        let mut moves = InlineBuffer::<256>::new();
        self.gen
            .gen_moves::<gen_type::All, _>(&self.board, &mut moves);

        if moves.len() == 0 {
            return None;
        }

        let color = match self.board.state.player {
            Player::White => -1,
            Player::Black => 1,
        };

        self.pv.clear();

        self.depth = 1;

        let mut best_move = Move::INVALID;
        let mut best_move_idx = 0;

        while self.depth <= Self::MAX_DEPTH {
            let mut upper = -Self::CHECKMATE_SCORE;
            let lower = Self::CHECKMATE_SCORE;
            let mut line = Line::new();

            for (idx, m) in moves.iter().copied().enumerate() {
                let undo = self.board.make_move(m);
                let value = -self.search(self.depth - 1, -lower, -upper, color, &mut line, &fc);
                self.board.unmake_move(undo);
                if value > upper {
                    self.pv.apply(m, &line);
                    upper = value;
                    best_move = m;
                    best_move_idx = idx;
                }
            }

            let cont = f(Info::Depth(self.depth as u16))
                .chain(f(Info::BestMove {
                    mov: best_move,
                    value: color * upper,
                }))
                .chain(f(Info::Nodes(self.nodes as usize)))
                .chain(f(Info::Pv(self.pv.get_pv().to_vec())))
                .chain(f(Info::Round));

            if cont == ShouldRun::Stop {
                break;
            }

            moves.swap(0, best_move_idx);

            self.depth += 1;
        }

        if best_move != Move::INVALID {
            Some(best_move)
        } else {
            None
        }
    }

    fn search<Fc: Fn() -> ShouldRun>(
        &mut self,
        depth: u8,
        mut lower: i32,
        mut upper: i32,
        color: i32,
        pline: &mut Line,
        fc: &Fc,
    ) -> i32 {
        if fc() == ShouldRun::Stop {
            return upper;
        }
        let pref_lower = lower;
        if let Some(x) = self.table.get(self.board.hash) {
            if x.depth > depth {
                match x.score {
                    TableScore::Exact(x) => {
                        return x;
                    }
                    TableScore::Min(x) => {
                        lower = lower.max(x);
                        if lower >= upper {
                            return x;
                        }
                    }
                    TableScore::Max(x) => {
                        upper = upper.min(x);
                        if lower >= upper {
                            return x;
                        }
                    }
                }
            }
        }

        if depth == 0 {
            return self.quiesce(lower, upper, color);
        }
        let mut buffer = InlineBuffer::<128>::new();
        let pos_info = self
            .gen
            .gen_moves::<gen_type::All, _>(&self.board, &mut buffer);
        if self.gen.drawn(&self.board, &pos_info) {
            return -self.contempt;
        }
        if buffer.len() == 0 {
            return -Self::CHECKMATE_SCORE;
        }

        let mut line = Line::new();
        let mut best_move = Move::INVALID;
        let mut v = i32::MIN;
        for m in buffer.iter().copied() {
            let undo = self.board.make_move(m);
            let score = -self.search(depth - 1, -upper, -lower, -color, &mut line, fc);
            self.board.unmake_move(undo);
            if score > v {
                pline.apply(m, &line);
                v = score;
                best_move = m;
            }
            lower = lower.max(score);
            if lower >= upper {
                break;
            }
        }
        self.table.set(TableValue {
            hash: self.board.hash,
            depth,
            move_: best_move,
            score: if v <= pref_lower {
                TableScore::Max(v)
            } else if v >= upper {
                TableScore::Min(v)
            } else {
                TableScore::Exact(v)
            },
        });
        v
    }

    fn quiesce(&mut self, mut lower: i32, upper: i32, color: i32) -> i32 {
        let value = color * self.eval_board();
        if value >= upper {
            return upper;
        }
        lower = lower.max(value);

        let mut buffer = InlineBuffer::<128>::new();
        self.gen
            .gen_moves::<gen_type::Captures, _>(&self.board, &mut buffer);
        for m in buffer.iter().copied() {
            let undo = self.board.make_move(m);
            let value = -self.quiesce(-upper, -lower, -color);
            self.board.unmake_move(undo);

            if value >= upper {
                return upper;
            }
            lower = lower.max(value);
        }
        lower
    }
}
