mod fen;
//mod gen;
mod bb;
pub use bb::BB;
mod extra_state;
mod render;
pub use extra_state::ExtraState;
mod gen;
pub use gen::{InlineBuffer, MoveBuffer, MoveGenerator};
mod square;
pub use square::Square;
mod mov;
pub use mov::Move;
pub mod eval;
pub mod hash;
use hash::Hasher;
mod util;

use std::{
    fmt::{self, Debug},
    iter::{ExactSizeIterator, Iterator},
    mem,
    ops::{Index, IndexMut},
};

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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Direction {
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

    #[inline(always)]
    pub fn player_pieces(black: bool) -> PieceIter {
        let add = if black { 6 } else { 0 };
        PieceIter {
            cur: Piece::WhiteKing as u8 + add,
            end: Piece::WhitePawn as u8 + add,
        }
    }

    #[inline(always)]
    pub fn player_promote_pieces(black: bool) -> PieceIter {
        let add = if black { 6 } else { 0 };
        PieceIter {
            cur: Piece::WhiteQueen as u8 + add,
            end: Piece::WhiteRook as u8 + add,
        }
    }

    #[inline(always)]
    pub fn player_king(black: bool) -> Self {
        let add = if black { 6 } else { 0 };
        unsafe { mem::transmute(Piece::WhiteKing as u8 + add) }
    }

    #[inline(always)]
    pub fn player_queen(black: bool) -> Self {
        let add = if black { 6 } else { 0 };
        unsafe { mem::transmute(Piece::WhiteQueen as u8 + add) }
    }

    #[inline(always)]
    pub fn player_rook(black: bool) -> Self {
        let add = if black { 6 } else { 0 };
        unsafe { mem::transmute(Piece::WhiteRook as u8 + add) }
    }

    #[inline(always)]
    pub fn player_bishop(black: bool) -> Self {
        let add = if black { 6 } else { 0 };
        unsafe { mem::transmute(Piece::WhiteBishop as u8 + add) }
    }

    #[inline(always)]
    pub fn player_knight(black: bool) -> Self {
        let add = if black { 6 } else { 0 };
        unsafe { mem::transmute(Piece::WhiteKnight as u8 + add) }
    }

    #[inline(always)]
    pub fn player_pawn(black: bool) -> Self {
        let add = if black { 6 } else { 0 };
        unsafe { mem::transmute(Piece::WhitePawn as u8 + add) }
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
    hash: u64,
}

impl fmt::Debug for UnmakeMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.mov)
    }
}

#[derive(Eq, PartialEq, Clone)]
pub struct Board {
    pieces: [BB; 12],

    pub state: ExtraState,

    moves: Vec<UnmakeMove>,

    hash: u64,
}

impl Board {
    pub const fn empty() -> Self {
        Board {
            pieces: [BB::empty(); 12],
            state: ExtraState::empty(),
            moves: Vec::new(),
            hash: 0,
        }
    }

    pub fn start_position() -> Self {
        let mut res = Board::empty();
        res.state.castle = ExtraState::BLACK_KING_CASTLE
            | ExtraState::BLACK_QUEEN_CASTLE
            | ExtraState::WHITE_KING_CASTLE
            | ExtraState::WHITE_QUEEN_CASTLE;
        res.state.black_turn = false;
        res.state.en_passant = u8::MAX;

        res[Piece::WhiteKing] = BB::E1;
        res[Piece::WhiteQueen] = BB::D1;
        res[Piece::WhiteBishop] = BB::C1 | BB::F1;
        res[Piece::WhiteKnight] = BB::B1 | BB::G1;
        res[Piece::WhiteRook] = BB::A1 | BB::H1;
        res[Piece::WhitePawn] = BB::RANK_2;
        res[Piece::BlackKing] = BB::E8;
        res[Piece::BlackQueen] = BB::D8;
        res[Piece::BlackBishop] = BB::C8 | BB::F8;
        res[Piece::BlackKnight] = BB::B8 | BB::G8;
        res[Piece::BlackRook] = BB::A8 | BB::H8;
        res[Piece::BlackPawn] = BB::RANK_7;
        res
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

    pub fn make_move(&mut self, m: Move, hasher: &Hasher) -> UnmakeMove {
        let white_turn = self.white_turn();
        let state = self.state;
        let hash = self.hash;
        self.state.black_turn = !self.state.black_turn;
        self.hash ^= hasher.black;
        self.hash ^= hasher.castle[self.state.castle as usize];

        match m {
            Move::Quiet { from, to, piece } => {
                self[piece] ^= BB::square(from) | BB::square(to);
                self.hash ^= hasher.pieces[piece][from];
                self.hash ^= hasher.pieces[piece][to];

                let mut castle_mask = 0;
                if from == Square::E1 {
                    castle_mask = ExtraState::WHITE_KING_CASTLE | ExtraState::WHITE_QUEEN_CASTLE
                }
                if from == Square::A1 {
                    castle_mask = ExtraState::WHITE_QUEEN_CASTLE
                }
                if from == Square::H1 {
                    castle_mask = ExtraState::WHITE_KING_CASTLE
                }
                if from == Square::E8 {
                    castle_mask = ExtraState::BLACK_KING_CASTLE | ExtraState::BLACK_QUEEN_CASTLE
                }
                if from == Square::A8 {
                    castle_mask = ExtraState::BLACK_QUEEN_CASTLE
                }
                if from == Square::H8 {
                    castle_mask = ExtraState::BLACK_KING_CASTLE
                }
                self.state.castle &= !castle_mask;
                self.hash ^= hasher.castle[self.state.castle as usize];

                let res = UnmakeMove {
                    mov: m,
                    taken: None,
                    state,
                    hash,
                };
                self.moves.push(res);
                res
            }
            Move::Capture {
                from,
                to,
                piece,
                taken,
            } => {
                let to_square = BB::square(to);
                self[piece] ^= BB::square(from) | to_square;
                self[taken] ^= to_square;
                self.hash ^= hasher.pieces[piece][from];
                self.hash ^= hasher.pieces[piece][to];
                self.hash ^= hasher.pieces[taken][to];

                let mut castle_mask = 0;
                if from == Square::E1 {
                    castle_mask = ExtraState::WHITE_KING_CASTLE | ExtraState::WHITE_QUEEN_CASTLE
                }
                if from == Square::A1 {
                    castle_mask = ExtraState::WHITE_QUEEN_CASTLE
                }
                if from == Square::H1 {
                    castle_mask = ExtraState::WHITE_KING_CASTLE
                }
                if from == Square::E8 {
                    castle_mask = ExtraState::BLACK_KING_CASTLE | ExtraState::BLACK_QUEEN_CASTLE
                }
                if from == Square::A8 {
                    castle_mask = ExtraState::BLACK_QUEEN_CASTLE
                }
                if from == Square::H8 {
                    castle_mask = ExtraState::BLACK_KING_CASTLE
                }

                if to == Square::A1 {
                    castle_mask |= ExtraState::WHITE_QUEEN_CASTLE
                }
                if to == Square::H1 {
                    castle_mask |= ExtraState::WHITE_KING_CASTLE
                }
                if to == Square::A8 {
                    castle_mask |= ExtraState::BLACK_QUEEN_CASTLE
                }
                if to == Square::H8 {
                    castle_mask |= ExtraState::BLACK_KING_CASTLE
                }
                self.state.castle &= !castle_mask;
                self.hash ^= hasher.castle[self.state.castle as usize];
                let res = UnmakeMove {
                    mov: m,
                    taken: None,
                    state,
                    hash,
                };
                self.moves.push(res);
                res
            }
            Move::Promote { to, from, promote } => {
                let piece = Piece::BlackPawn.flip(white_turn);
                self[piece] ^= BB::square(from);
                self[promote] |= BB::square(to);

                self.hash ^= hasher.pieces[piece][from];
                self.hash ^= hasher.pieces[promote][to];
                self.hash ^= hasher.castle[self.state.castle as usize];

                let res = UnmakeMove {
                    mov: m,
                    taken: None,
                    state,
                    hash,
                };
                self.moves.push(res);
                res
            }
            Move::PromoteCapture {
                to,
                from,
                promote,
                taken,
            } => {
                let piece = Piece::BlackPawn.flip(white_turn);
                self[piece] ^= BB::square(from);
                self[promote] |= BB::square(to);
                self[taken] ^= BB::square(to);

                self.hash ^= hasher.pieces[piece][from];
                self.hash ^= hasher.pieces[promote][to];
                self.hash ^= hasher.pieces[taken][to];

                let mut castle_mask = 0;
                if to == Square::A1 {
                    castle_mask = ExtraState::BLACK_QUEEN_CASTLE
                }
                if to == Square::H1 {
                    castle_mask = ExtraState::BLACK_KING_CASTLE
                }
                if to == Square::A8 {
                    castle_mask = ExtraState::WHITE_QUEEN_CASTLE
                }
                if to == Square::H8 {
                    castle_mask = ExtraState::WHITE_KING_CASTLE
                }
                self.state.castle &= !castle_mask;
                self.hash ^= hasher.castle[self.state.castle as usize];

                let res = UnmakeMove {
                    mov: m,
                    taken: None,
                    state,
                    hash,
                };
                self.moves.push(res);
                res
            }
            Move::Castle { king } => {
                let rook_move = if king {
                    if white_turn {
                        (Square::H1, Square::F1)
                    } else {
                        (Square::H8, Square::F8)
                    }
                } else {
                    if white_turn {
                        (Square::A1, Square::D1)
                    } else {
                        (Square::A8, Square::D8)
                    }
                };

                let king_move = if king {
                    if white_turn {
                        (Square::E1, Square::G1)
                    } else {
                        (Square::E8, Square::G8)
                    }
                } else {
                    if white_turn {
                        (Square::E1, Square::C1)
                    } else {
                        (Square::E8, Square::C8)
                    }
                };

                let p_rook = Piece::BlackRook.flip(white_turn);
                let p_king = Piece::BlackKing.flip(white_turn);

                self[p_rook] ^= BB::square(rook_move.0) | BB::square(rook_move.1);
                self[p_king] ^= BB::square(king_move.0) | BB::square(king_move.1);

                self.hash ^= hasher.pieces[p_rook][rook_move.0];
                self.hash ^= hasher.pieces[p_rook][rook_move.1];
                self.hash ^= hasher.pieces[p_king][king_move.0];
                self.hash ^= hasher.pieces[p_king][king_move.1];

                let castle_mask = if white_turn {
                    ExtraState::BLACK_QUEEN_CASTLE | ExtraState::BLACK_KING_CASTLE
                } else {
                    ExtraState::WHITE_QUEEN_CASTLE | ExtraState::WHITE_KING_CASTLE
                };
                self.state.castle &= castle_mask;

                self.hash ^= hasher.castle[self.state.castle as usize];
                let res = UnmakeMove {
                    mov: m,
                    taken: None,
                    state,
                    hash,
                };
                self.moves.push(res);
                res
            }
            _ => todo!(),
        }
    }

    pub fn unmake_move(&mut self, mov: UnmakeMove, hasher: &Hasher) {
        assert_eq!(self.moves.pop(), Some(mov));
        self.hash ^= hasher.castle[self.state.castle as usize];
        self.hash ^= hasher.castle[mov.state.castle as usize];
        self.hash ^= hasher.black;
        self.state = mov.state;
        match mov.mov {
            Move::Quiet { from, to, piece } => {
                self[piece] ^= BB::square(from) | BB::square(to);
                self.hash ^= hasher.pieces[piece][from];
                self.hash ^= hasher.pieces[piece][to];
            }
            Move::Capture {
                from,
                to,
                piece,
                taken,
            } => {
                self[piece] ^= BB::square(from) | BB::square(to);
                self[taken] ^= BB::square(to);
                self.hash ^= hasher.pieces[piece][from];
                self.hash ^= hasher.pieces[piece][to];
                self.hash ^= hasher.pieces[taken][to];
            }
            Move::Castle { king } => {
                let white_turn = !self.state.black_turn;
                let rook_move = if king {
                    if white_turn {
                        (Square::H1, Square::F1)
                    } else {
                        (Square::H8, Square::F8)
                    }
                } else {
                    if white_turn {
                        (Square::A1, Square::D1)
                    } else {
                        (Square::A8, Square::D8)
                    }
                };

                let king_move = if king {
                    if white_turn {
                        (Square::E1, Square::G1)
                    } else {
                        (Square::E8, Square::G8)
                    }
                } else {
                    if white_turn {
                        (Square::E1, Square::C1)
                    } else {
                        (Square::E8, Square::C8)
                    }
                };

                let p_rook = Piece::BlackRook.flip(white_turn);
                let p_king = Piece::BlackKing.flip(white_turn);

                self[p_rook] ^= BB::square(rook_move.0) | BB::square(rook_move.1);
                self[p_king] ^= BB::square(king_move.0) | BB::square(king_move.1);
                self.hash ^= hasher.pieces[p_rook][rook_move.0];
                self.hash ^= hasher.pieces[p_rook][rook_move.1];
                self.hash ^= hasher.pieces[p_king][king_move.0];
                self.hash ^= hasher.pieces[p_king][king_move.1];
            }
            Move::Promote { promote, from, to } => {
                let pawn = Piece::player_pawn(self.state.black_turn);
                self[promote] ^= BB::square(to);
                self[pawn] |= BB::square(from);
                self.hash ^= hasher.pieces[promote][to];
                self.hash ^= hasher.pieces[pawn][from];
            }
            Move::PromoteCapture {
                promote,
                from,
                to,
                taken,
            } => {
                let pawn = Piece::player_pawn(self.state.black_turn);
                self[promote] ^= BB::square(to);
                self[taken] ^= BB::square(to);
                self[pawn] |= BB::square(from);
                self.hash ^= hasher.pieces[promote][to];
                self.hash ^= hasher.pieces[pawn][from];
                self.hash ^= hasher.pieces[taken][to];
            }
            _ => todo!(),
        }
        assert_eq!(self.hash, mov.hash);
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
        !self.state.black_turn
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
            .field("hash", &self.hash)
            .finish()
    }
}
