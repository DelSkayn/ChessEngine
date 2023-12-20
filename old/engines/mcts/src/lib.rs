#![allow(dead_code)]

use chess_core::{
    uci::{
        engine::{Engine, EngineSignal, SearchConfig, SearchResult},
        Version,
    },
    Board, Move, Square,
};

pub struct Node {
    m: Move,
    pending: Vec<Move>,
    children: Vec<Node>,
    score: f32,
    simulations: u32,
}

pub struct Config {
    playouts: usize,
    max_playout_depth: usize,
    exploration: f32,
}

pub struct Mcts {
    tree: Node,
    config: Config,
    board: Board,
}

impl Engine for Mcts {
    const NAME: &'static str = "Mcts Random Playout";
    const AUTHOR: &'static str = "Mees Delzenne";

    fn version() -> Option<chess_core::uci::Version> {
        let major = env!("CARGO_PKG_VERSION_MAJOR").parse().ok()?;
        let minor = env!("CARGO_PKG_VERSION_MINOR").parse().ok()?;
        let patch = env!("CARGO_PKG_VERSION_PATCH").parse().ok()?;
        Some(Version {
            major,
            minor,
            patch,
        })
    }

    fn options() -> Vec<chess_core::uci::OptionMsg> {
        vec![]
    }

    fn set_option(&mut self, _name: String, _value: Option<String>) {}

    fn new_position(&mut self, board: chess_core::Board) {
        self.board = board;
    }

    fn make_move(&mut self, r#move: chess_core::Move) {
        self.board.make_move(r#move);
    }

    fn search(&mut self, _config: SearchConfig, _signal: EngineSignal) -> SearchResult {
        self.tree = Node {
            m: Move::normal(Square::A1, Square::A1),
            pending: Vec::new(),
            children: Vec::new(),
            score: 0.0,
            simulations: 0,
        };

        todo!()
    }
}
