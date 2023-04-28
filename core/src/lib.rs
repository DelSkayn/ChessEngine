#![allow(dead_code)]

//mod gen;
pub mod bb;
pub mod board;
pub mod engine;
#[deprecated]
pub mod eval;
mod extra_state;
pub mod gen;
pub mod gen2;
pub mod hash;
mod mov;
mod piece;
mod render;
mod square;
pub mod uci;
pub mod util;

use board::MoveChain;
pub use board::{Board, UnmakeMove};
pub use extra_state::ExtraState;
pub use mov::{Move, Promotion};
pub use piece::Piece;
pub use square::Square;

/// Enumr representing a player.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Player {
    White,
    Black,
}

impl Player {
    /// Returns the other player
    pub fn flip(self) -> Self {
        match self {
            Player::Black => Player::White,
            Player::White => Player::Black,
        }
    }
}

/// A direction on the board with the side with the black pieces being north.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Direction {
    NW = 0,
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
}

impl Direction {
    /// Creates a direction from u8
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Direction::NW),
            1 => Some(Direction::N),
            2 => Some(Direction::NE),
            3 => Some(Direction::E),
            4 => Some(Direction::SE),
            5 => Some(Direction::S),
            6 => Some(Direction::SW),
            7 => Some(Direction::W),
            _ => None,
        }
    }

    /// Returns the board square index offset of the direction.
    #[inline(always)]
    pub const fn as_offset(self) -> i8 {
        match self {
            Direction::NW => 7,
            Direction::N => 8,
            Direction::NE => 9,
            Direction::E => 1,
            Direction::SE => -7,
            Direction::S => -8,
            Direction::SW => -9,
            Direction::W => -1,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum GameOutcome {
    Won(Player),
    Drawn,
    None,
}

impl GameOutcome {
    pub fn from_board<C: MoveChain>(b: &Board<C>, gen: &gen::MoveGenerator) -> Self {
        let info = gen.gen_info(b);
        if gen.is_board_drawn(b, &info) {
            return GameOutcome::Drawn;
        }
        if gen.is_checkmate(b, &info) {
            return GameOutcome::Won(b.state.player.flip());
        }
        return GameOutcome::None;
    }
}
