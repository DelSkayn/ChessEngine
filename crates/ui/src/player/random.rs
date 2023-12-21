use super::Player;
use crate::{game::PlayedMove, RenderBoard};
use chess_core::{
    gen::{gen_type, MoveGenerator},
    Move,
};
use rand::{thread_rng, Rng};
use std::time::{Duration, Instant};

pub struct RandomPlayer {
    move_gen: MoveGenerator,
    possible_moves: Vec<Move>,
    time: Instant,
}

impl RandomPlayer {
    pub fn new() -> Self {
        RandomPlayer {
            move_gen: MoveGenerator::new(),
            possible_moves: Vec::new(),
            time: Instant::now(),
        }
    }
}

impl Player for RandomPlayer {
    fn start_turn(&mut self, board: &RenderBoard) {
        self.possible_moves.clear();
        self.move_gen
            .gen_moves::<gen_type::All, _, _>(&board.board, &mut self.possible_moves);
        self.time = Instant::now()
    }

    fn update(&mut self, board: &mut RenderBoard) -> PlayedMove {
        if self.possible_moves.is_empty() || self.time.elapsed() < Duration::from_secs_f32(0.5) {
            return PlayedMove::Didnt;
        }

        let pick = thread_rng().gen_range(0..self.possible_moves.len());
        let mov = self.possible_moves[pick];

        board.clear_highlight();

        board.highlight(mov.from(), mov.to());
        board.make_move(self.possible_moves[pick]);

        PlayedMove::Move
    }
}
