//! A interface for an chess engine

use crate::{board2::Board, Move};
use std::collections::HashMap;

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

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum ShouldRun {
    Continue,
    Stop,
}

impl ShouldRun {
    pub fn chain(self, other: Self) -> ShouldRun {
        if other == ShouldRun::Stop {
            ShouldRun::Stop
        } else {
            self
        }
    }
}

pub trait Engine: 'static {
    const AUTHOR: &'static str = "Mees Delzenne";
    const NAME: &'static str;

    /// Run the search
    fn go<F: FnMut(Info) -> ShouldRun, Fc: Fn() -> ShouldRun>(
        &mut self,
        f: F,
        fc: Fc,
    ) -> Option<Move>;

    /// Set the board
    fn set_board(&mut self, board: Board);

    /// Make a move on the current board
    fn make_move(&mut self, m: Move);

    /// Start a new game
    /// Only a hint
    fn new_game(&mut self) {}

    /// Get the options
    fn options(&self) -> HashMap<String, OptionKind>;

    /// Set an option
    fn set_option(&mut self, name: String, value: OptionValue);
}
