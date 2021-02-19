mod fen;
//mod gen;
mod bb;
pub use bb::BB;
mod extra_state;
mod render;
pub use extra_state::ExtraState;
mod gen;
pub use gen::MoveGenerator;

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

#[derive(Debug)]
pub struct Move {
    piece: u8,
    promote: u8,
    from: u8,
    to: u8,
}
pub trait Player {
    const MY_KING: u8;
    const MY_QUEEN: u8;
    const MY_BISHOP: u8;
    const MY_KNIGHT: u8;
    const MY_ROOK: u8;
    const MY_PAWN: u8;

    const OP_KING: u8;
    const OP_QUEEN: u8;
    const OP_BISHOP: u8;
    const OP_KNIGHT: u8;
    const OP_ROOK: u8;
    const OP_PAWN: u8;
}

#[derive(Eq, PartialEq, Clone, Copy)]
pub struct Board {
    pieces: [BB; 12],

    state: ExtraState,
}

impl Board {
    const WHITE_KING: usize = 0;
    const WHITE_QUEEN: usize = 1;
    const WHITE_BISHOP: usize = 2;
    const WHITE_KNIGHT: usize = 3;
    const WHITE_ROOK: usize = 4;
    const WHITE_PAWN: usize = 5;

    const BLACK_KING: usize = 6;
    const BLACK_QUEEN: usize = 7;
    const BLACK_BISHOP: usize = 8;
    const BLACK_KNIGHT: usize = 9;
    const BLACK_ROOK: usize = 10;
    const BLACK_PAWN: usize = 11;

    pub const fn empty() -> Self {
        Board {
            pieces: [BB::empty(); 12],
            state: ExtraState::empty(),
        }
    }

    pub fn flip(mut self) -> Self {
        for i in 0..12 {
            self.pieces[i] = self.pieces[i].flip()
        }
        for i in 0..6 {
            self.pieces[i] ^= self.pieces[i + 6];
            self.pieces[i + 6] ^= self.pieces[i];
            self.pieces[i] ^= self.pieces[i + 6];
        }
        return self;
    }

    pub fn make_move(mut self, m: Move) -> Self {
        for i in 6..12 {
            self.pieces[i] &= !(1 << m.to);
        }
        dbg!(m.from);
        self.pieces[m.piece as usize] &= !(1 << m.from);
        if m.promote != 0 {
            self.pieces[m.promote as usize] |= 1 << m.to;
        } else {
            self.pieces[m.piece as usize] |= 1 << m.to;
        }
        //Casteling
        if m.piece % 6 == 0 && (m.from as i8 - m.to as i8).abs() == 2 {
            match m.to {
                2 => self.pieces[Self::WHITE_ROOK] ^= 0b1001,
                6 => self.pieces[Self::WHITE_ROOK] ^= 0b10100000,
                58 => self.pieces[Self::BLACK_ROOK] ^= 0b10001 << 56,
                62 => self.pieces[Self::BLACK_ROOK] ^= 0b101 << 61,
                _ => panic!(),
            }
        }
        self
    }
}

impl Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Board")
            .field("white_king", &self.pieces[Board::WHITE_KING])
            .field("white_queen", &self.pieces[Board::WHITE_QUEEN])
            .field("white_bishop", &self.pieces[Board::WHITE_BISHOP])
            .field("white_knight", &self.pieces[Board::WHITE_KNIGHT])
            .field("white_rook", &self.pieces[Board::WHITE_ROOK])
            .field("white_pawn", &self.pieces[Board::WHITE_PAWN])
            .field("black_king", &self.pieces[Board::BLACK_KING])
            .field("black_queen", &self.pieces[Board::BLACK_QUEEN])
            .field("black_bishop", &self.pieces[Board::BLACK_BISHOP])
            .field("black_knight", &self.pieces[Board::BLACK_KNIGHT])
            .field("black_rook", &self.pieces[Board::BLACK_ROOK])
            .field("black_pawn", &self.pieces[Board::BLACK_PAWN])
            .field("state", &self.state)
            .finish()
    }
}
