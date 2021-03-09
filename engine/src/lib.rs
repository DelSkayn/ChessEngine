mod fen;
//mod gen;
mod bb;
pub use bb::BB;
mod extra_state;
mod render;
pub use extra_state::ExtraState;
mod gen;
pub use gen::MoveGenerator;
mod square;
pub use square::Square;
mod mov;
pub use mov::Move;

use std::{
    fmt::{self, Debug},
    ops::{Index, IndexMut},
};
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

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Piece {
    WhiteKing = 0,
    WhiteQueen,
    WhiteBishop,
    WhiteKnight,
    WhiteRook,
    WhitePawn,
    BlackKing,
    BlackQueen,
    BlackBishop,
    BlackKnight,
    BlackRook,
    BlackPawn,
}

impl Piece {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Piece::WhiteKing,
            1 => Piece::WhiteQueen,
            2 => Piece::WhiteBishop,
            3 => Piece::WhiteKnight,
            4 => Piece::WhiteRook,
            5 => Piece::WhitePawn,
            6 => Piece::BlackKing,
            7 => Piece::BlackQueen,
            8 => Piece::BlackBishop,
            9 => Piece::BlackKnight,
            10 => Piece::BlackRook,
            11 => Piece::BlackPawn,
            x => panic!("invalid number for piece: {}", x),
        }
    }

    pub fn white(self) -> bool {
        (self as u8) < 6
    }
}

impl Board {
    /*
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
    */

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
        self.state = self.state.flip();
        return self;
    }

    pub fn make_move(mut self, m: Move) -> Self {
        match m {
            Move::Simple { to, from, piece } => {
                self[piece] ^= BB::square(from) | BB::square(to);
                for i in 6..12 {
                    self.pieces[i] &= !BB::square(to);
                }
            }
            Move::Promote { to, from, promote } => {
                self[Piece::WhitePawn] &= !BB::square(from);
                for i in 6..12 {
                    self.pieces[i] &= !BB::square(to);
                }
                self[promote] |= BB::square(to);
            }
            Move::Castle { king } => {
                if king {
                    self[Piece::WhiteRook] ^= BB::square(Square::H1) | BB::square(Square::F1);
                    self[Piece::WhiteKing] ^= BB::square(Square::E1) | BB::square(Square::G1);
                } else {
                    self[Piece::WhiteRook] ^= BB::square(Square::A1) | BB::square(Square::D1);
                    self[Piece::WhiteKing] ^= BB::square(Square::E1) | BB::square(Square::C1);
                }
            }
            _ => todo!(),
        }
        self.state = self.state.make_move();

        self
    }

    pub fn on(&self, square: Square) -> Option<Piece> {
        let bb = BB::square(square);
        for i in 0..12 {
            let piece = Piece::from_u8(i);
            if (self[piece] & bb).any() {
                return Some(piece);
            }
        }
        None
    }

    pub fn white_turn(&self) -> bool {
        self.state.white_move()
    }
}

impl Index<Piece> for Board {
    type Output = BB;
    fn index(&self, index: Piece) -> &Self::Output {
        unsafe { self.pieces.get_unchecked(index as usize) }
    }
}

impl IndexMut<Piece> for Board {
    fn index_mut(&mut self, index: Piece) -> &mut Self::Output {
        unsafe { self.pieces.get_unchecked_mut(index as usize) }
    }
}

impl Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Board")
            .field("white_king", &self[Piece::WhiteKing])
            .field("white_queen", &self[Piece::WhiteQueen])
            .field("white_bishop", &self[Piece::WhiteBishop])
            .field("white_knight", &self[Piece::WhiteKnight])
            .field("white_rook", &self[Piece::WhiteRook])
            .field("white_pawn", &self[Piece::WhitePawn])
            .field("black_king", &self[Piece::BlackKing])
            .field("black_queen", &self[Piece::BlackQueen])
            .field("black_bishop", &self[Piece::BlackBishop])
            .field("black_knight", &self[Piece::BlackKnight])
            .field("black_rook", &self[Piece::BlackRook])
            .field("black_pawn", &self[Piece::BlackPawn])
            .field("state", &self.state)
            .finish()
    }
}
