use std::collections::HashMap;

use anyhow::Result;
use chess_core::{
    board::EndChain,
    engine::{Engine, EngineControl, OptionKind},
    gen::{gen_type, MoveGenerator},
    Board, Move,
};
use chess_uci::Uci;
use rand::Rng;

pub struct Random {
    board: Board,
    gen: MoveGenerator,
}

impl Random {
    pub fn new() -> Self {
        Random {
            board: Board::start_position(EndChain),
            gen: MoveGenerator::new(),
        }
    }
}

impl<C: EngineControl> Engine<C> for Random {
    const NAME: &'static str = "Random";

    fn go(
        &mut self,
        _: C,
        _: Option<std::time::Duration>,
        _: chess_core::engine::EngineLimit,
    ) -> Option<Move> {
        let mut moves = Vec::new();
        self.gen
            .gen_moves::<gen_type::All, _, _>(&self.board, &mut moves);
        if moves.len() == 0 {
            return None;
        }
        Some(moves[rand::thread_rng().gen_range(0..moves.len())])
    }

    fn set_board(&mut self, board: Board) {
        self.board = board;
    }

    fn make_move(&mut self, m: Move) {
        self.board.make_move(m);
    }

    fn options(&self) -> HashMap<String, OptionKind> {
        HashMap::new()
    }

    fn set_option(&mut self, _: String, _: chess_core::engine::OptionValue) {}
}

fn main() -> Result<()> {
    Uci::new(Random::new()).start()
}
