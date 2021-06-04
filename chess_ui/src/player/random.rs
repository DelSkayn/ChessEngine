use super::Player;
use crate::{game::PlayedMove, RenderBoard};
use chess_core::{gen2::MoveGenerator, Move};
use ggez::event::MouseButton;
use rand::{thread_rng, Rng};

pub struct RandomPlayer {
    move_gen: MoveGenerator,
    possible_moves: Vec<Move>,
}

impl RandomPlayer {
    pub fn new() -> Self {
        RandomPlayer {
            move_gen: MoveGenerator::new(),
            possible_moves: Vec::new(),
        }
    }
}

impl Player for RandomPlayer {
    fn start_turn(&mut self, board: &RenderBoard) {
        self.possible_moves.clear();
        self.move_gen
            .gen_moves(&board.board, &mut self.possible_moves);
    }

    fn mouse_button_up_event(
        &mut self,
        button: MouseButton,
        _x: f32,
        _y: f32,
        board: &mut RenderBoard,
    ) -> PlayedMove {
        if button != MouseButton::Left {
            return PlayedMove::Didnt;
        }

        if self.possible_moves.is_empty() {
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
