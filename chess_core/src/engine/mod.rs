//! A interface for an chess engine

use crate::{Board, Move};
use std::{collections::HashMap, time::Duration};

mod thread;
pub use thread::{EngineThread, Response, ThreadController};

#[derive(Debug)]
pub enum Info {
    // A best move found
    BestMove { mov: Move, value: i32 },
    // Engine has moved on to new depth
    Depth(u16),
    // New Principle variation
    Pv(Vec<Move>),
    // Amount of nodes searched
    Nodes(usize),
    // Nodes searched per second
    NodesPerSec(f32),
    // Amount of transposition table hits
    TransHit(usize),
    // Engine completed a round
    Round,
    Debug(String),
}

#[derive(Clone)]
pub enum OptionKind {
    Check,
    Spin {
        default: i32,
        min: Option<i32>,
        max: Option<i32>,
    },
    Combo(Vec<String>),
    Button,
    String,
}

#[derive(Clone)]
pub enum OptionValue {
    Check(bool),
    Spin(i32),
    Combo(usize),
    Button,
    String(String),
}

pub trait EngineControl: Default + 'static {
    fn should_stop(&self) -> bool;

    fn info(&self, info: Info);
}

#[derive(Default)]
pub struct NoControl;

impl EngineControl for NoControl {
    fn should_stop(&self) -> bool {
        false
    }

    fn info(&self, _: Info) {}
}

#[derive(Default)]
pub struct EngineLimit {
    pub depth: Option<u32>,
    pub nodes: Option<u64>,
    pub time: Option<Duration>,
}

impl EngineLimit {
    pub fn time(d: Duration) -> Self {
        EngineLimit {
            time: Some(d),
            ..Default::default()
        }
    }

    pub fn nodes(d: u64) -> Self {
        EngineLimit {
            nodes: Some(d),
            ..Default::default()
        }
    }

    pub fn depth(d: u32) -> Self {
        EngineLimit {
            depth: Some(d),
            ..Default::default()
        }
    }

    pub fn none() -> Self {
        Default::default()
    }

    pub fn or(&self, other: EngineLimit) -> Self {
        EngineLimit {
            depth: self.depth.or(other.depth),
            nodes: self.nodes.or(other.nodes),
            time: self.time.or(other.time),
        }
    }
}

pub trait Engine<C: EngineControl>: 'static {
    const AUTHOR: &'static str = "Mees Delzenne";
    const NAME: &'static str;

    /// Run the search
    fn go(&mut self, control: C, time_left: Option<Duration>, limit: EngineLimit) -> Option<Move>;

    /*
    /// Run the search
    fn go<F: FnMut(Info) -> ShouldRun, Fc: Fn() -> ShouldRun>(
        &mut self,
        f: F,
        fc: Fc,
    ) -> Option<Move>;
    */

    /// Set the board
    fn set_board(&mut self, board: Board);

    /// Make a move on the current board
    fn make_move(&mut self, m: Move);

    /// Start a new game
    /// Only a hint
    fn new_game(&mut self) {}

    /// Get the options
    fn options(&self) -> HashMap<String, OptionKind> {
        HashMap::new()
    }

    /// Set an option
    fn set_option(&mut self, _: String, _: OptionValue) {}
}
