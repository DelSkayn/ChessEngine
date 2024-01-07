mod eval;
mod hash_table;
mod search;

use std::io;

use common::board::Board;
use hash_table::HashTable;
use move_gen::{types::gen_type, InlineBuffer, MoveGenerator};
use uci::{
    engine::{self, Engine, RunContext, SearchResult},
    req::GoRequest,
};

pub struct AlphaBeta {
    move_gen: MoveGenerator,
    hash: HashTable,
    board: Board,
    nodes_searched: u64,
    hash_collisions: u64,
    moves_played_hash: Vec<u64>,
    contempt: i32,
}

impl AlphaBeta {
    pub const DEFAULT_HASH_SIZE: usize = 128 * 1024 * 1024;

    pub fn new() -> Self {
        let board = Board::start_position();
        AlphaBeta {
            move_gen: MoveGenerator::new(),
            hash: HashTable::new_size(Self::DEFAULT_HASH_SIZE),
            nodes_searched: 0,
            hash_collisions: 0,
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

    fn new_game(&mut self) {
        self.hash.reset();
        self.board = Board::start_position();
        self.moves_played_hash.clear();
        self.nodes_searched = 0;
    }

    fn position(&mut self, mut board: Board, moves: &[uci::UciMove]) {
        self.moves_played_hash.clear();
        self.moves_played_hash.push(board.hash);

        let mut diverges = false;

        for (idx, m) in moves.iter().enumerate() {
            let mut moves = InlineBuffer::new();
            self.move_gen.gen_moves::<gen_type::All>(&board, &mut moves);
            let Some(m) = m.to_move(moves.as_slice()) else {
                eprintln!("move '{}' was not valid! ignoring", m);
                break;
            };
            board.make_move(m);
            self.moves_played_hash.push(self.board.hash);

            if idx == self.moves_played_hash.len() - 1 {
                diverges = !board.is_equal(&self.board)
            }
            if idx >= self.moves_played_hash.len() {
                self.hash.advance_move()
            }
        }

        if diverges {
            eprintln!("game diverged, reseting hash");
            self.hash.reset();
        }

        self.board = board;
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
