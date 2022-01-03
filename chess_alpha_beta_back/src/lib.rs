#![allow(dead_code)]

use chess_core::{
    board::{Board as BaseBoard, EndChain, HashChain},
    engine::{Engine, Info, OptionKind, OptionValue, ShouldRun},
    gen::MoveGenerator,
    Move,
};

mod eval;
mod hash;
mod search;
mod sort;
use search::Line;

use std::collections::HashMap;

type Board = BaseBoard<HashChain<EndChain>>;

pub struct AlphaBeta {
    contempt: i32,
    board: Board,
    table: hash::HashTable,
    gen: MoveGenerator,
    pv: Line,
    nodes: u64,
    table_hit: u64,
    depth: u8,
}

impl AlphaBeta {
    pub fn new() -> Self {
        AlphaBeta {
            contempt: 100,
            board: Board::start_position(HashChain::new()),
            table: hash::HashTable::new(16 * 1024),
            gen: MoveGenerator::new(),
            pv: Line::new(),
            nodes: 0,
            table_hit: 0,
            depth: 0,
        }
    }
}

impl Engine for AlphaBeta {
    const NAME: &'static str = "Alpha Beta 1";

    fn go<F: FnMut(Info) -> ShouldRun, Fc: Fn() -> ShouldRun>(
        &mut self,
        f: F,
        fc: Fc,
    ) -> Option<Move> {
        self.nodes = 0;

        self.go_search(f, fc)
    }

    fn make_move(&mut self, m: Move) {
        self.board.make_move(m);
        println!("HASH: {:x}", self.board.chain.hash);
    }

    fn options(&self) -> HashMap<String, OptionKind> {
        [
            (
                "contempt".to_string(),
                OptionKind::Spin {
                    default: 100,
                    max: Some(900),
                    min: Some(-100),
                },
            ),
            (
                "Hash".to_string(),
                OptionKind::Spin {
                    default: 16,
                    min: Some(1),
                    max: Some(1024 * 4),
                },
            ),
        ]
        .iter()
        .cloned()
        .collect()
    }

    fn set_option(&mut self, name: String, value: OptionValue) {
        match name.as_str() {
            "Hash" => {
                if let OptionValue::Spin(x) = value {
                    if x > 1 && x < 1024 * 4 {
                        self.table = hash::HashTable::new(x as usize * 1024)
                    }
                }
            }
            "contempt" => {
                if let OptionValue::Spin(x) = value {
                    self.contempt = x;
                }
            }
            _ => {}
        }
    }

    fn new_game(&mut self) {
        self.board = Board::start_position(HashChain::new());
    }

    fn set_board(&mut self, board: BaseBoard) {
        self.board.copy_position(&board);
        println!("SET_POSITION");
    }
}
