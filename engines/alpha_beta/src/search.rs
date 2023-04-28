use std::{mem::MaybeUninit, time::Instant};

use chess_core::{
    gen::{gen_type, InlineBuffer, MoveList},
    uci::{engine::EngineSignal, InfoMsg, ScoreKind, UciMove},
    Move, Player,
};

use crate::AlphaBeta;

pub struct PvBuffer<const DEPTH: usize> {
    len: [u8; DEPTH],
    pv: [[MaybeUninit<Move>; DEPTH]; DEPTH],
}

impl<const DEPTH: usize> PvBuffer<DEPTH> {
    pub fn new() -> Self {
        PvBuffer {
            len: [0u8; DEPTH],
            pv: [[MaybeUninit::uninit(); DEPTH]; DEPTH],
        }
    }

    pub fn clear(&mut self) {
        self.len = [0u8; DEPTH];
    }

    #[inline]
    pub fn pv(&self, depth: usize) -> &[Move] {
        assert!(depth < DEPTH);
        unsafe {
            let ptr = (&self.pv[depth]) as *const MaybeUninit<Move>;
            std::slice::from_raw_parts(ptr as *const Move, self.len[depth] as usize)
        }
    }

    #[inline]
    pub fn pv_mut(&mut self, depth: usize) -> &mut [Move] {
        assert!(depth < DEPTH);
        unsafe {
            let ptr = (&mut self.pv[depth]) as *mut MaybeUninit<Move>;
            std::slice::from_raw_parts_mut(ptr as *mut Move, self.len[depth] as usize)
        }
    }

    // Copy over the line of the lower depth to the current depth
    pub fn write_line_from_lower(&mut self, depth: usize, m: Move) {
        assert!(depth < DEPTH - 1);
        self.pv[depth][0].write(m);
        let next_depth = (depth + 1) as usize;
        let len = self.len[next_depth];
        unsafe {
            let src = self.pv[next_depth].as_ptr();
            let dst = src.sub(DEPTH - 1);
            std::ptr::copy_nonoverlapping(src as *const Move, dst as *mut Move, len.into());
        }
        self.len[depth] = len + 1;
    }
}

pub struct SearchInfo {
    max_depth: u8,
    signal: EngineSignal,
    start: Instant,
}

impl AlphaBeta {
    pub const SCORE_INF: i32 = i32::MAX;
    pub const SCORE_CHECKMATE: i32 = i32::MAX - 1;
    pub const MAX_DEPTH: u8 = 50;

    pub fn search_root(&mut self, signal: EngineSignal) -> Option<Move> {
        self.limit.depth = self.limit.depth.min(Self::MAX_DEPTH);
        let start = Instant::now();

        let mut root_moves = Vec::new();
        self.gen
            .gen_moves::<gen_type::All, _, _>(&self.board, &mut root_moves);

        if root_moves.is_empty() {
            return None;
        }

        let mut best_move = root_moves[0];

        let mut info = SearchInfo {
            max_depth: 1,
            signal,
            start,
        };

        let color: i32 = if self.board.state.player == Player::White {
            1
        } else {
            -1
        };

        self.pv.clear();

        while info.max_depth <= self.limit.depth
            && self.limit.time > start.elapsed()
            && !info.signal.should_stop()
        {
            let start_iteration = Instant::now();

            let mut best_score = -Self::SCORE_INF;
            let mut best_move_depth = root_moves[0];

            for &m in &root_moves {
                let undo = self.board.make_move(m);
                let score = -self.alpha_beta(-Self::SCORE_INF, -best_score, 1, -color, &info);
                self.board.unmake_move(undo);

                // If should stop is enabled the score generated
                // is probably not correct.
                if info.signal.should_stop() {
                    return Some(best_move);
                }

                if score > best_score {
                    best_score = score;
                    best_move_depth = m;
                    self.pv.write_line_from_lower(0, m);
                }
            }

            best_move = best_move_depth;
            for idx in 0..root_moves.len() {
                if root_moves[idx] == best_move_depth {
                    root_moves.swap(0, idx);
                    break;
                }
            }

            info.signal.info(vec![
                InfoMsg::Depth(info.max_depth as u32),
                InfoMsg::Score {
                    value: best_score * color,
                    kind: ScoreKind::Cp,
                },
                InfoMsg::Nodes(self.info.nodes),
                InfoMsg::Nps((self.info.nodes as f64 / start.elapsed().as_secs_f64()) as u64),
                InfoMsg::Pv(self.pv.pv(0).iter().copied().map(UciMove::from).collect()),
            ]);

            info.max_depth += 1;

            // If we have less time left than the last iteration times the amount of root moves
            // then stop searching. The next iteration takes roughly that amount of time so if we
            // have less time left we will probably need to stop early making the time spent useless.
            //
            // Pruning techniques might lower that time taken so we half the next time estimate.
            if self
                .limit
                .time
                .checked_sub(start.elapsed())
                .unwrap_or_default()
                <= start_iteration.elapsed() * root_moves.len() as u32 / 2
            {
                break;
            }
        }

        return Some(best_move);
    }

    // Alpha is lowerbound
    // Beta is upperbound
    pub fn alpha_beta(
        &mut self,
        mut alpha: i32,
        mut beta: i32,
        depth: u8,
        color: i32,
        info: &SearchInfo,
    ) -> i32 {
        if depth == info.max_depth {
            // score is high if the position is good for the current player.
            let score = self.quicense(alpha, beta, color);
            if self.info.nodes >= self.limit.nodes {
                info.signal.stop();
            }
            return score;
        }

        let mut moves = InlineBuffer::<256>::new();
        self.gen
            .gen_moves::<gen_type::All, _, _>(&self.board, &mut moves);

        let pv_move = self.pv.pv(0).get(depth as usize).copied();

        if let Some(x) = pv_move {
            for idx in 0..moves.len() {
                if moves.get(idx) == x {
                    moves.swap(0, idx);
                    break;
                }
            }
        }

        let old_beta = beta;

        for (idx, m) in moves.iter().enumerate() {
            if info.start.elapsed() > self.limit.time {
                info.signal.stop();
            }

            if info.signal.should_stop() {
                return 0;
            }

            let undo = self.board.make_move(m);
            // score is low if the move for the next player is high and vice versa.
            let score = -self.alpha_beta(-beta, -alpha, depth + 1, color * -1, &info);
            if old_beta != beta && score >= beta && idx != 0 && depth < info.max_depth - 1 {
                alpha = -self.alpha_beta(-old_beta, -alpha, depth + 1, color * -1, &info);
                self.pv.write_line_from_lower(depth as usize, m);
            }

            self.board.unmake_move(undo);

            if score > alpha {
                alpha = score;
                self.pv.write_line_from_lower(depth as usize, m);
            }
            if alpha >= old_beta {
                return alpha;
            }

            beta = alpha + 1;
        }

        return alpha;
    }

    pub fn quicense(&mut self, mut alpha: i32, beta: i32, color: i32) -> i32 {
        let info = self.gen.gen_info(&self.board);
        let value = color * self.eval_board(&info);

        if value >= beta {
            return beta;
        }

        alpha = alpha.max(value);

        let mut moves = InlineBuffer::<128>::new();
        self.gen
            .gen_moves_info::<gen_type::Captures, _, _>(&self.board, &info, &mut moves);

        for m in moves.iter() {
            let undo = self.board.make_move(m);
            let score = -self.quicense(-beta, -alpha, -color);
            self.board.unmake_move(undo);

            if score >= beta {
                return beta;
            }

            //if score < alpha - QUEEN_VALUE {
            //   return alpha;
            //}

            alpha = alpha.max(score);
        }

        return alpha;
    }
}
