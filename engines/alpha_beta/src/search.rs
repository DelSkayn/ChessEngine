use common::{Move, Player};
use move_gen::{types::gen_type, InlineBuffer};
use std::time::{Duration, Instant};
use uci::{
    engine::RunContext,
    req::GoRequest,
    resp::{ResponseBound, ResponseInfo, ResponseScore},
};

use super::AlphaBeta;

impl AlphaBeta {
    pub const CHECKMATE_SCORE: i32 = i32::MAX - 1000;

    pub fn search(&mut self, settings: &GoRequest, ctx: RunContext<'_>) -> Move {
        self.nodes_searched = 0;
        let search_start = Instant::now();
        let deadline = self.deadline(&search_start, settings);

        let mut root_moves = InlineBuffer::new();
        self.move_gen
            .gen_moves::<gen_type::All>(&self.board, &mut root_moves);

        if root_moves.is_empty() {
            return Move::NULL;
        }

        let mut depth = 0;
        let mut total_best_move = None;

        'depth_loop: loop {
            eprint!("depth: {depth}, ");
            let iteration_start = Instant::now();

            let mut best_score = -i32::MAX;
            let mut best_move = None;
            let mut best_move_idx = None;

            for (idx, m) in root_moves.iter().enumerate() {
                let undo = self.board.make_move(m);
                let score = if self.would_repeat(self.board.hash) {
                    self.score_sign() * self.contempt
                } else {
                    self.moves_played_hash.push(self.board.hash);
                    let score = -self.search_moves(-i32::MAX, -best_score, depth);
                    self.moves_played_hash.pop();
                    score
                };
                //eprintln!("{m} = {score} hash {}", self.board.hash);
                self.board.unmake_move(undo);

                if iteration_start
                    + (iteration_start.elapsed() / (idx as u32 + 1) * root_moves.len() as u32)
                    > deadline
                {
                    break 'depth_loop;
                }

                if score > best_score {
                    best_move = Some(m);
                    best_score = score;
                    best_move_idx = Some(idx);
                }
            }

            if RunContext::should_stop() {
                break;
            }

            root_moves.swap(0, best_move_idx.unwrap() as u8);

            let best_move = best_move.unwrap();

            eprintln!(
                "score: {}, move {}, nodes {}",
                self.score_sign() * best_score,
                best_move,
                self.nodes_searched
            );

            ctx.info(ResponseInfo::Nodes(self.nodes_searched));
            ctx.info(ResponseInfo::Depth(depth.into()));
            ctx.info(ResponseInfo::CurrMove(best_move.into()));
            ctx.info(ResponseInfo::Score(ResponseScore {
                mate: None,
                cp: Some((self.score_sign() * best_score) as i64),
                bound: ResponseBound::Exact,
            }));

            total_best_move = Some(best_move);

            if best_score.abs() == Self::CHECKMATE_SCORE {
                break;
            }

            // if the current iteration took more time than is left we can assume we don't can't
            // finish the next iterator since it most likely takes longer.
            if Instant::now() + iteration_start.elapsed() * 2 > deadline {
                break;
            }

            if depth == 255 {
                break;
            }
            depth += 1;
        }

        total_best_move.unwrap_or(Move::NULL)
    }

    pub fn search_moves(&mut self, mut alpha: i32, beta: i32, depth: u8) -> i32 {
        if depth == 0 {
            return self.quiesce(alpha, beta);
        }

        let info = self.move_gen.gen_info(&self.board);

        if self.move_gen.drawn_by_rule(&self.board, &info) {
            return self.contempt;
        }

        let mut buffer = InlineBuffer::new();
        self.move_gen
            .gen_moves::<gen_type::All>(&self.board, &mut buffer);
        let buffer = buffer;

        if buffer.is_empty() {
            if self.move_gen.checked_king(&self.board, &info) {
                return -Self::CHECKMATE_SCORE;
            }
            return 0;
        }

        if RunContext::should_stop() {
            return i32::MAX;
        }

        for m in buffer.iter() {
            let undo = self.board.make_move(m);
            let score = if self.would_repeat(self.board.hash) {
                self.score_sign() * self.contempt
            } else {
                self.moves_played_hash.push(self.board.hash);
                let score = -self.search_moves(-beta, -alpha, depth - 1);
                self.moves_played_hash.pop();
                score
            };
            self.board.unmake_move(undo);
            if score >= beta {
                return beta;
            };
            alpha = alpha.max(score);
        }

        alpha
    }

    pub fn quiesce(&mut self, mut alpha: i32, beta: i32) -> i32 {
        self.nodes_searched += 1;

        let info = self.move_gen.gen_info(&self.board);

        let score = if self.move_gen.check_mate(&self.board, &info) {
            -Self::CHECKMATE_SCORE
        } else {
            self.score_sign() * self.eval()
        };

        alpha = alpha.max(score);

        let mut moves = InlineBuffer::new();
        self.move_gen
            .gen_moves::<gen_type::Captures>(&self.board, &mut moves);
        let moves = moves;

        for m in moves.iter() {
            let undo = self.board.make_move(m);
            let score = -self.quiesce(-beta, -alpha);
            self.board.unmake_move(undo);

            if score >= beta {
                return beta;
            }
            alpha = alpha.max(score);
        }
        alpha
    }

    fn score_sign(&self) -> i32 {
        if self.board.state.player == Player::White {
            1
        } else {
            -1
        }
    }

    fn would_repeat(&mut self, hash: u64) -> bool {
        self.moves_played_hash
            .iter()
            .rev()
            .take(self.board.state.move_clock as usize)
            .skip(1)
            .step_by(2)
            .any(|x| *x == hash)
    }

    fn deadline(&self, start: &Instant, settings: &GoRequest) -> Instant {
        if settings.infinite {
            return *start + Duration::from_secs(60 * 60 * 24 * 356);
        }

        if let Some(time) = settings.movetime {
            return *start + Duration::from_millis(time);
        }

        if let Some(wtime) = settings.wtime {
            let inc = settings.winc.unwrap_or(0) as i64;

            if self.board.state.player == Player::White {
                let time = (wtime / 20) + inc;
                return *start + Duration::from_millis(time.max(0) as u64);
            }
        }

        if let Some(btime) = settings.btime {
            let inc = settings.binc.unwrap_or(0) as i64;

            if self.board.state.player == Player::Black {
                let time = (btime / 20) + inc;
                return *start + Duration::from_millis(time.max(0) as u64);
            }
        }

        *start + Duration::from_secs(60 * 60 * 24 * 356)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use common::{board::Board, Promotion, Square};
    use uci::engine::Engine;

    #[test]
    fn test_checkmate() {
        let fen = "r1bqkbnr/pppp1ppp/2n5/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR b KQkq - 3 4";
        let mut engine = AlphaBeta::new();
        engine.board = Board::from_fen(fen).unwrap();

        let m = Move::new(
            Square::from_name("g8").unwrap(),
            Square::from_name("f6").unwrap(),
            Move::TYPE_NORMAL,
            common::Promotion::Queen,
        );

        RunContext::force_run();

        engine.board.make_move(m);
        let score = -engine.search_moves(-i32::MAX, i32::MAX, 1);
        assert_eq!(score, -AlphaBeta::CHECKMATE_SCORE);
    }

    #[test]
    fn test_checkmate_2() {
        let fen = "5Q2/6np/p6k/8/4q1pK/7P/8/8 w - - 0 1";
        let mut engine = AlphaBeta::new();
        engine.board = Board::from_fen(fen).unwrap();

        RunContext::force_run();

        let mut root_moves = InlineBuffer::new();
        engine
            .move_gen
            .gen_moves::<gen_type::All>(&engine.board, &mut root_moves);
        let root_moves = root_moves;
        let mut best_score = -i32::MAX;
        let old_board = engine.board.clone();
        for m in root_moves.iter() {
            let undo = engine.board.make_move(m);
            let score = -engine.search_moves(-i32::MAX, -best_score, 4);
            println!("{m} {score}");
            engine.board.unmake_move(undo);
            assert!(old_board.is_equal(&engine.board));
            if score > best_score {
                best_score = score;
            }
        }
        assert_eq!(best_score, AlphaBeta::CHECKMATE_SCORE);
    }

    #[test]
    fn test_capture_1() {
        let fen = "r1bqkbnr/ppppp1Qp/8/8/1n6/8/PPPPPPPP/RNB1KBNR b KQkq - 0 1";

        let mut engine = AlphaBeta::new();
        engine.board = Board::from_fen(fen).unwrap();
        let mut root_moves = InlineBuffer::new();

        RunContext::force_run();

        engine
            .move_gen
            .gen_moves::<gen_type::All>(&engine.board, &mut root_moves);
        let root_moves = root_moves;
        let mut best_score = -i32::MAX;
        let mut best_move = None;
        let old_board = engine.board.clone();
        for m in root_moves.iter() {
            let undo = engine.board.make_move(m);
            let score = -engine.search_moves(-i32::MAX, -best_score, 3);
            println!("{m} {score}");
            engine.board.unmake_move(undo);
            assert!(old_board.is_equal(&engine.board));
            if score > best_score {
                best_score = score;
                best_move = Some(m);
            }
        }
        assert_eq!(best_move.unwrap().to(), Square::from_name("g7").unwrap());
    }

    #[test]
    fn test_weird_move_1() {
        let fen = "r1bqk2r/ppppbppp/2n1pn2/1B4B1/3PP3/2P5/PP1N1PPP/R2QK1NR b KQkq - 0 1";

        let mut engine = AlphaBeta::new();
        engine.board = Board::from_fen(fen).unwrap();

        RunContext::force_run();

        let m = Move::new(
            Square::from_name("c6").unwrap(),
            Square::from_name("d4").unwrap(),
            Move::TYPE_NORMAL,
            Promotion::Queen,
        );

        engine.board.make_move(m);
        let score = -engine.search_moves(-i32::MAX, i32::MAX, 0);
        println!("{m} {score}");

        assert!(score < 100);
    }

    #[test]
    fn test_repetition() {
        let fen = "8/5N2/R6p/1pk3p1/5p2/5K1r/8/8 w - - 7 8";

        let mut engine = AlphaBeta::new();
        engine.position(
            Board::from_fen(fen).unwrap(),
            &[
                "f3g4".parse().unwrap(),
                "h3h4".parse().unwrap(),
                "g4f3".parse().unwrap(),
                "h4h3".parse().unwrap(),
            ],
        );

        engine.board.make_move(Move::from_name("f3g4").unwrap());
        assert!(engine.would_repeat(engine.board.hash));

        engine.moves_played_hash.insert(0, 0);
        assert!(engine.would_repeat(engine.board.hash));
    }
}
