mod fen;
//mod gen;
mod bb;
pub use bb::BB;
mod extra_state;
mod render;
pub use extra_state::ExtraState;

use std::fmt::{self, Debug};

#[derive(Eq, PartialEq, Clone, Copy)]
pub struct Square(pub u8);

impl Debug for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let file = self.0 % 8;
        let rank = self.0 / 8;
        let file_name = ['h', 'g', 'f', 'e', 'd', 'c', 'b', 'a'];
        writeln!(f, "{}{}", file_name[file as usize], 8 - rank)
    }
}

struct Move {
    from: u8,
    to: u8,
}

#[derive(Eq, PartialEq)]
pub struct Board {
    pieces: [BB; 12],

    state: ExtraState,
}

impl Board {
    const WHITE_KING: u8 = 0;
    const WHITE_QUEEN: u8 = 1;
    const WHITE_BISHOP: u8 = 2;
    const WHITE_KNIGHT: u8 = 3;
    const WHITE_ROOK: u8 = 4;
    const WHITE_PAWN: u8 = 5;

    const BLACK_KING: u8 = 6;
    const BLACK_QUEEN: u8 = 7;
    const BLACK_BISHOP: u8 = 8;
    const BLACK_KNIGHT: u8 = 9;
    const BLACK_ROOK: u8 = 10;
    const BLACK_PAWN: u8 = 11;

    pub const fn empty() -> Self {
        Board {
            pieces: [BB::empty(); 12],
            state: ExtraState::empty(),
        }
    }
}

impl Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Board")
            .field("white_king", &self.pieces[Board::WHITE_KING as usize])
            .field("white_queen", &self.pieces[Board::WHITE_QUEEN as usize])
            .field("white_bishop", &self.pieces[Board::WHITE_BISHOP as usize])
            .field("white_knight", &self.pieces[Board::WHITE_KNIGHT as usize])
            .field("white_rook", &self.pieces[Board::WHITE_ROOK as usize])
            .field("white_pawn", &self.pieces[Board::WHITE_PAWN as usize])
            .field("black_king", &self.pieces[Board::BLACK_KING as usize])
            .field("black_queen", &self.pieces[Board::BLACK_QUEEN as usize])
            .field("black_bishop", &self.pieces[Board::BLACK_BISHOP as usize])
            .field("black_knight", &self.pieces[Board::BLACK_KNIGHT as usize])
            .field("black_rook", &self.pieces[Board::BLACK_ROOK as usize])
            .field("black_pawn", &self.pieces[Board::BLACK_PAWN as usize])
            .field("state", &self.state)
            .finish()
    }
}
