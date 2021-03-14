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
pub mod eval;

use std::{
    fmt::{self, Debug},
    iter::{ExactSizeIterator, Iterator},
    mem,
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

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(u8)]
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
    pub fn to(self, end: Piece) -> PieceIter {
        PieceIter {
            cur: self as u8,
            end: end as u8,
        }
    }

    pub fn flip(self, v: bool) -> Self {
        let val = (self as u8 + (v as u8 * 6)) % 12;
        unsafe { mem::transmute(val) }
    }
}

pub struct PieceIter {
    cur: u8,
    end: u8,
}

impl Iterator for PieceIter {
    type Item = Piece;

    fn next(&mut self) -> Option<Piece> {
        if self.cur > self.end {
            return None;
        }
        let res = unsafe { std::mem::transmute(self.cur) };
        self.cur += 1;
        Some(res)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let res = self.len();
        (res, Some(res))
    }
}

impl ExactSizeIterator for PieceIter {
    fn len(&self) -> usize {
        (self.end - self.cur + 1) as usize
    }
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

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct UnmakeMove {
    mov: Move,
    taken: Option<Piece>,
    state: ExtraState,
}

impl fmt::Debug for UnmakeMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.mov)
    }
}

#[derive(Eq, PartialEq, Clone)]
pub struct Board {
    pieces: [BB; 12],

    state: ExtraState,

    moves: Vec<UnmakeMove>,
}

impl Board {
    pub const fn empty() -> Self {
        Board {
            pieces: [BB::empty(); 12],
            state: ExtraState::empty(),
            moves: Vec::new(),
        }
    }

    pub fn assert_valid(&self) {
        assert_eq!(
            self[Piece::WhiteKing].count(),
            1,
            "{:?}\n Missing a white king",
            self
        );
        assert_eq!(
            self[Piece::BlackKing].count(),
            1,
            "{:?}\n Missing a black king",
            self
        );
        for pa in Piece::WhiteKing.to(Piece::BlackPawn) {
            for pb in Piece::WhiteKing.to(pa) {
                if pa == pb {
                    continue;
                }
                assert!(
                    (self[pa] & self[pb]).none(),
                    "{:?}\n Overlap in bitboards: {:?} {:?}",
                    self,
                    pa,
                    pb
                );
            }
        }
    }

    pub fn flip(mut self) -> Self {
        for i in 0..6 {
            self.pieces[i].flip();
            self.pieces[i + 6].flip();
            self.pieces[i] ^= self.pieces[i + 6];
            self.pieces[i + 6] ^= self.pieces[i];
            self.pieces[i] ^= self.pieces[i + 6];
        }
        self.state = self.state.flip();
        return self;
    }

    pub fn make_move(&mut self, m: Move) -> UnmakeMove {
        let white_turn = self.white_turn();

        match m {
            Move::Simple { to, from, piece } => {
                let mut taken = None;
                let state = self.state;
                let to_square = BB::square(to);
                for p in Piece::WhiteQueen
                    .flip(white_turn)
                    .to(Piece::WhitePawn.flip(white_turn))
                {
                    if (self[p] & to_square).any() {
                        taken = Some(p);
                    }
                    self[p] &= !to_square;
                }
                self[piece] ^= BB::square(from) | to_square;
                self.state &= !(ExtraState::fill(to == Square::H8) & ExtraState::BLACK_KING_CASTLE);
                self.state &=
                    !(ExtraState::fill(to == Square::A8) & ExtraState::BLACK_QUEEN_CASTLE);
                self.state &= !(ExtraState::fill(to == Square::H1) & ExtraState::WHITE_KING_CASTLE);
                self.state &=
                    !(ExtraState::fill(to == Square::A1) & ExtraState::WHITE_QUEEN_CASTLE);
                self.state = self.state.make_move();
                let res = UnmakeMove {
                    mov: m,
                    taken,
                    state,
                };
                self.moves.push(res);
                self.assert_valid();
                res
            }
            Move::Promote { to, from, promote } => {
                let mut taken = None;
                let state = self.state;
                let to_square = BB::square(to);
                for p in Piece::WhiteQueen
                    .flip(white_turn)
                    .to(Piece::WhitePawn.flip(white_turn))
                {
                    if (self[p] & to_square).any() {
                        taken = Some(p);
                    }
                    self[p] &= !to_square;
                }
                self[Piece::BlackPawn.flip(white_turn)] &= !BB::square(from);
                self.state &= !(ExtraState::fill(to == Square::H8) & ExtraState::BLACK_KING_CASTLE);
                self.state &=
                    !(ExtraState::fill(to == Square::A8) & ExtraState::BLACK_QUEEN_CASTLE);
                self.state &= !(ExtraState::fill(to == Square::H1) & ExtraState::WHITE_KING_CASTLE);
                self.state &=
                    !(ExtraState::fill(to == Square::A1) & ExtraState::WHITE_QUEEN_CASTLE);
                self[promote] |= BB::square(to);
                self.state = self.state.make_move();
                let res = UnmakeMove {
                    mov: m,
                    taken,
                    state,
                };
                self.moves.push(res);
                self.assert_valid();
                res
            }
            Move::Castle { king } => {
                let state = self.state;
                if white_turn {
                    self[Piece::WhiteRook] ^= BB::fill(king)
                        & (BB::square(Square::H1) | BB::square(Square::F1))
                        | BB::fill(!king) & (BB::square(Square::A1) | BB::square(Square::D1));

                    self[Piece::WhiteKing] ^= BB::fill(king)
                        & (BB::square(Square::E1) | BB::square(Square::G1))
                        | BB::fill(!king) & (BB::square(Square::E1) | BB::square(Square::C1));
                    self.state &= !(ExtraState::BLACK_KING_CASTLE | ExtraState::BLACK_QUEEN_CASTLE);
                } else {
                    self[Piece::BlackRook] ^= BB::fill(king)
                        & (BB::square(Square::H8) | BB::square(Square::F8))
                        | BB::fill(!king) & (BB::square(Square::A8) | BB::square(Square::D8));

                    self[Piece::BlackKing] ^= BB::fill(king)
                        & (BB::square(Square::E8) | BB::square(Square::G8))
                        | BB::fill(!king) & (BB::square(Square::E8) | BB::square(Square::C8));
                    self.state &= !(ExtraState::WHITE_KING_CASTLE | ExtraState::WHITE_QUEEN_CASTLE);
                }
                self.state = self.state.make_move();
                let res = UnmakeMove {
                    mov: m,
                    taken: None,
                    state,
                };
                self.moves.push(res);
                self.assert_valid();
                res
            }
            _ => todo!(),
        }
    }

    pub fn unmake_move(&mut self, mov: UnmakeMove) {
        assert_eq!(self.moves.pop(), Some(mov));
        self.state = mov.state;
        match mov.mov {
            Move::Simple { from, to, piece } => {
                self[piece] ^= BB::square(from) | BB::square(to);
                if let Some(x) = mov.taken {
                    self[x] |= BB::square(to);
                }
            }
            Move::Castle { king } => {
                if self.state.white_move() {
                    self[Piece::WhiteRook] ^= BB::fill(king)
                        & (BB::square(Square::H1) | BB::square(Square::F1))
                        | BB::fill(!king) & (BB::square(Square::A1) | BB::square(Square::D1));

                    self[Piece::WhiteKing] ^= BB::fill(king)
                        & (BB::square(Square::E1) | BB::square(Square::G1))
                        | BB::fill(!king) & (BB::square(Square::E1) | BB::square(Square::C1));
                } else {
                    self[Piece::BlackRook] ^= BB::fill(king)
                        & (BB::square(Square::H8) | BB::square(Square::F8))
                        | BB::fill(!king) & (BB::square(Square::A8) | BB::square(Square::D8));

                    self[Piece::BlackKing] ^= BB::fill(king)
                        & (BB::square(Square::E8) | BB::square(Square::G8))
                        | BB::fill(!king) & (BB::square(Square::E8) | BB::square(Square::C8));
                }
            }
            Move::Promote { promote, from, to } => {
                self[promote] &= !BB::square(to);
                let white_turn = self.state.white_move();
                self[Piece::BlackPawn.flip(white_turn)] |= BB::square(from);
                if let Some(x) = mov.taken {
                    self[x] |= BB::square(to);
                }
            }
            _ => todo!(),
        }
    }

    pub fn on(&self, square: Square) -> Option<Piece> {
        let bb = BB::square(square);
        for piece in Piece::WhiteKing.to(Piece::BlackPawn) {
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
            .field("moves", &self.moves)
            .finish()
    }
}
