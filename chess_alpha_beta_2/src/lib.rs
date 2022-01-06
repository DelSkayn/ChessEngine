#![allow(dead_code)]

use chess_core::{
    board::{Board as BaseBoard, EndChain, HashChain},
    engine::{Engine, EngineControl, EngineLimit, OptionKind, OptionValue},
    gen::MoveGenerator,
    Move,
};

mod eval;
mod hash;
mod search;
mod sort;
use search::Line;

use std::{
    cell::Cell,
    collections::HashMap,
    time::{Duration, Instant},
};

type Board = BaseBoard<HashChain<EndChain>>;

pub struct TimeLimit {
    start: Instant,
    limit: Duration,
    nodes_searched: u64,
    exceeded: Cell<bool>,
}

impl TimeLimit {
    const WAIT_NODES: u64 = 10_000;

    pub fn limit(limit: Duration) -> Self {
        TimeLimit {
            start: Instant::now(),
            limit,
            nodes_searched: 0,
            exceeded: Cell::new(false),
        }
    }

    fn check_time(&self, nodes: u64) -> bool {
        if nodes > self.nodes_searched + Self::WAIT_NODES {
            let res = self.start.elapsed() > self.limit;
            self.exceeded.set(res);
            res
        } else {
            false
        }
    }

    #[inline]
    pub fn should_stop(&self, nodes: u64) -> bool {
        self.exceeded.get() || self.check_time(nodes)
    }
}

pub struct AlphaBeta<C> {
    contempt: i32,
    board: Board,
    table: hash::HashTable,
    gen: MoveGenerator,
    pv: Line,
    nodes: u64,
    table_hit: u64,
    depth: u8,
    control: C,
    limits: EngineLimit,
    time_limit: Option<TimeLimit>,
}

impl<C: EngineControl> AlphaBeta<C> {
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
            control: C::default(),
            limits: EngineLimit::none(),
            time_limit: None,
        }
    }
}

impl<C: EngineControl> Engine<C> for AlphaBeta<C> {
    const NAME: &'static str = "AlphaBeta 2";

    fn go(
        &mut self,
        control: C,
        time_left: Option<std::time::Duration>,
        limit: chess_core::engine::EngineLimit,
    ) -> Option<Move> {
        self.control = control;
        self.limits = limit;

        let time_limit = match (self.limits.time, time_left) {
            (None, None) => None,
            (Some(x), None) => Some(x),
            (None, Some(x)) => Some(x / 30),
            (Some(a), Some(b)) => Some(a.min(b / 30)),
        };

        self.time_limit = time_limit.map(TimeLimit::limit);

        self.go_search()
    }

    fn make_move(&mut self, m: Move) {
        self.board.make_move(m);
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
    }
}
