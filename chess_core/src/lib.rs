#![allow(dead_code)]

mod fen;
//mod gen;
mod bb;
mod board;
pub mod engine;
pub mod eval;
mod extra_state;
//pub mod gen;
pub mod gen2;
pub mod gen3;
pub mod hash;
mod mov;
mod piece;
mod render;
mod square;
pub mod uci;
pub mod util;

pub use bb::BB;
pub use board::{Board, UnmakeMove};
pub use extra_state::ExtraState;
pub use mov::Move;
pub use piece::Piece;
pub use square::Square;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Player {
    White,
    Black,
}

impl Player {
    pub fn flip(self) -> Self {
        match self {
            Player::Black => Player::White,
            Player::White => Player::Black,
        }
    }
}

/// A direction on the board with the side with the black pieces being north.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Direction::NW,
            1 => Direction::N,
            2 => Direction::NE,
            3 => Direction::E,
            4 => Direction::SE,
            5 => Direction::S,
            6 => Direction::SW,
            7 => Direction::W,
            _ => panic!(),
        }
    }

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

/*
impl fmt::Debug for UnmakeMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.mov)
    }
}
*/
