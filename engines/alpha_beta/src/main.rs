mod eval;
mod search;

use std::io;

use common::board::Board;
use move_gen::{types::gen_type, InlineBuffer, MoveGenerator};
use uci::{
    engine::{self, Engine, RunContext, SearchResult},
    req::GoRequest,
};

pub struct AlphaBeta {
    move_gen: MoveGenerator,
    board: Board,
    nodes_searched: u64,
    moves_played_hash: Vec<u64>,
    contempt: i32,
}

impl AlphaBeta {
    pub fn new() -> Self {
        let board = Board::start_position();
        AlphaBeta {
            move_gen: MoveGenerator::new(),
            nodes_searched: 0,
            moves_played_hash: vec![board.hash],
            board,
            contempt: 0,
        }
    }
}

impl Default for AlphaBeta {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine for AlphaBeta {
    const NAME: &'static str = concat!("AlphaBeta Engine ", env!("CARGO_PKG_VERSION"));
    const AUTHOR: &'static str = env!("CARGO_PKG_AUTHORS");

    fn new() -> Self {
        AlphaBeta::new()
    }

    fn position(&mut self, board: Board, moves: &[uci::UciMove]) {
        self.board = board;
        self.moves_played_hash.clear();
        self.moves_played_hash.push(self.board.hash);
        for m in moves {
            let mut moves = InlineBuffer::new();
            self.move_gen
                .gen_moves::<gen_type::All>(&self.board, &mut moves);
            let Some(m) = m.to_move(moves.as_slice()) else {
                break;
            };
            self.board.make_move(m);
            self.moves_played_hash.push(self.board.hash);
        }
    }

    fn go(&mut self, settings: &GoRequest, context: RunContext<'_>) -> engine::SearchResult {
        let r#move = self.search(settings, context);
        SearchResult {
            r#move,
            ponder: None,
        }
    }
}

fn main() -> Result<(), io::Error> {
    engine::run::<AlphaBeta>()
}
