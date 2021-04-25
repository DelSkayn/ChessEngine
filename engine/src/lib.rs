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
use util::BoardArray;

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

    fn to_char(self) -> char{
    match self {
        Piece::WhiteKing => 'K',
        Piece::BlackKing => 'k',
        Piece::WhiteQueen => 'Q',
        Piece::BlackQueen => 'q',
        Piece::WhiteRook => 'R',
        Piece::BlackRook => 'r',
        Piece::WhiteBishop => 'B',
        Piece::BlackBishop => 'b',
        Piece::WhiteKnight => 'N',
        Piece::BlackKnight => 'n',
        Piece::WhitePawn => 'A',
        Piece::BlackPawn => 'a',
    }
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

#[derive(Clone, Copy, Eq, PartialEq,Debug)]
pub struct UnmakeMove {
    mov: Move,
    taken: Option<Piece>,
    state: ExtraState,
    hash: u64,
}

/*
impl fmt::Debug for UnmakeMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.mov)
    }
}
*/

#[derive(Eq, PartialEq, Clone)]
pub struct Board {
    pieces: [BB; 12],
    pub state: ExtraState,
    squares: BoardArray<Option<Piece>>,

    //moves: Vec<UnmakeMove>,

    hash: u64,
}

impl Board {
    pub const fn empty() -> Self {
        Board {
            pieces: [BB::empty(); 12],
            squares: BoardArray::new_array([None;64]),
            state: ExtraState::empty(),
            //moves: Vec::new(),
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
        res.squares[Square::E1] = Some(Piece::WhiteKing);
        res[Piece::WhiteQueen] = BB::D1;
        res.squares[Square::D1] = Some(Piece::WhiteQueen);
        res[Piece::WhiteBishop] = BB::C1 | BB::F1;
        res.squares[Square::C1] = Some(Piece::WhiteBishop);
        res.squares[Square::F1] = Some(Piece::WhiteBishop);
        res[Piece::WhiteKnight] = BB::B1 | BB::G1;
        res.squares[Square::B1] = Some(Piece::WhiteKnight);
        res.squares[Square::G1] = Some(Piece::WhiteKnight);
        res[Piece::WhiteRook] = BB::A1 | BB::H1;
        res.squares[Square::A1] = Some(Piece::WhiteRook);
        res.squares[Square::H1] = Some(Piece::WhiteRook);
        res[Piece::WhitePawn] = BB::RANK_2;
        for f in 0..8{
            res.squares[Square::from_file_rank(f,1)] = Some(Piece::WhitePawn);
        }

        res[Piece::BlackKing] = BB::E8;
        res.squares[Square::E8] = Some(Piece::BlackKing);
        res[Piece::BlackQueen] = BB::D8;
        res.squares[Square::D8] = Some(Piece::BlackQueen);
        res[Piece::BlackBishop] = BB::C8 | BB::F8;
        res.squares[Square::C8] = Some(Piece::BlackBishop);
        res.squares[Square::F8] = Some(Piece::BlackBishop);
        res[Piece::BlackKnight] = BB::B8 | BB::G8;
        res.squares[Square::B8] = Some(Piece::BlackKnight);
        res.squares[Square::G8] = Some(Piece::BlackKnight);
        res[Piece::BlackRook] = BB::A8 | BB::H8;
        res.squares[Square::A8] = Some(Piece::BlackRook);
        res.squares[Square::H8] = Some(Piece::BlackRook);
        res[Piece::BlackPawn] = BB::RANK_7;
        for f in 0..8{
            res.squares[Square::from_file_rank(f,6)] = Some(Piece::BlackPawn);
        }

        res
    }

    pub fn is_equal(&self,other: &Self) -> bool{
        if self.hash != other.hash{
            println!("hash not equal");
            return false;
        }

        for p in Piece::WhiteKing.to(Piece::BlackPawn){
            if self[p] != other[p]{
                println!("{:?}:{:?} != {:?}",p,self[p],other[p]);
                return false;
            }
        }

        if self.state != other.state{
            println!("state not equal");
            return false;
        }

        if self.squares != other.squares{
            println!("squaers not equal");
            return false;
        }

        true
    }

    pub fn is_valid(&self) -> bool {
        let mut res = true;

        if self[Piece::WhiteKing].count() != 1{
            res = false;
            eprintln!("Wrong number of white kings\n{:?}",self[Piece::WhiteKing]);
        }
        if self[Piece::BlackKing].count() != 1{
            res = false;
            eprintln!("Wrong number of black kings\n{:?}",self[Piece::BlackKing]);
        }
        for pa in Piece::WhiteKing.to(Piece::BlackPawn) {
            for pb in Piece::WhiteKing.to(pa) {
                if pa == pb {
                    continue;
                }
                if !(self[pa] & self[pb]).none(){
                eprintln!(
                    "Overlap in bitboards: {:?} {:?}\n{:?}{:?}",
                    pa,
                    pb,
                    self[pa],
                    self[pb]
                );
                res = false;
                }
            }
        }

        for s in 0..64{
            let s = Square::new(s);
            if let Some(x) = self.squares[s]{
                if !(self[x] & BB::square(s)).any() {
                    eprintln!("mailbox-bitboard mismatch, Square {} should contain {:?} but bitboard does not:\n{:?}",s,x,self[x]);
                    res = false;
                }
            }else{
                let sb = BB::square(s);
                for p in Piece::WhiteKing.to(Piece::BlackPawn){
                    if !(self[p] & sb).none(){
                        eprintln!("mailbox-bitboard mismatch, Square {} should be empty but contains {:?} on bitboard\n{:?}",s,p,self[p]);
                        res = false;
                    }
                }
            }
        }

        res
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
        let state = self.state;
        let hash = self.hash;
        self.hash ^= hasher.black;
        self.hash ^= hasher.castle[self.state.castle as usize];

        let from = m.from();
        let to = m.to();
        let ty = m.ty();
        if self.squares[from].is_none(){
            println!("{:?}",self);
            println!("{}",self);
        }
        let piece = self.squares[from].unwrap();
        let taken = self.squares[to];

        self.squares[from] = None;
        let mut castle_mask = 0;

        if ty == Move::TYPE_CASTLE{
            let king = piece;
            self[king] ^= BB::square(from) | BB::square(to);
            self.squares[to] = Some(king);
            self.hash ^= hasher.pieces[king][from];
            self.hash ^= hasher.pieces[king][to];

            match to {
                Square::C1 => {
                    self.squares[Square::A1] = None;
                    self.squares[Square::D1] = Some(Piece::WhiteRook);
                    self[Piece::WhiteRook] ^= BB::A1 | BB::D1;
                    self.hash ^= hasher.pieces[Piece::WhiteRook][Square::A1];
                    self.hash ^= hasher.pieces[Piece::WhiteRook][Square::D1];
                    castle_mask = ExtraState::WHITE_KING_CASTLE | ExtraState::WHITE_QUEEN_CASTLE;
                }
                Square::G1 => {
                    self.squares[Square::H1] = None;
                    self.squares[Square::F1] = Some(Piece::WhiteRook);
                    self[Piece::WhiteRook] ^= BB::H1 | BB::F1;
                    self.hash ^= hasher.pieces[Piece::WhiteRook][Square::H1];
                    self.hash ^= hasher.pieces[Piece::WhiteRook][Square::F1];
                    castle_mask = ExtraState::WHITE_KING_CASTLE | ExtraState::WHITE_QUEEN_CASTLE;
                }
                Square::C8 => {
                    self.squares[Square::A8] = None;
                    self.squares[Square::D8] = Some(Piece::BlackRook);
                    self[Piece::BlackRook] ^= BB::A8 | BB::D8;
                    self.hash ^= hasher.pieces[Piece::BlackRook][Square::A8];
                    self.hash ^= hasher.pieces[Piece::BlackRook][Square::D8];
                    castle_mask = ExtraState::BLACK_KING_CASTLE | ExtraState::BLACK_QUEEN_CASTLE;
                }
                Square::G8 => {
                    self.squares[Square::H8] = None;
                    self.squares[Square::F8] = Some(Piece::BlackRook);
                    self[Piece::BlackRook] ^= BB::H8 | BB::F8;
                    self.hash ^= hasher.pieces[Piece::BlackRook][Square::H8];
                    self.hash ^= hasher.pieces[Piece::BlackRook][Square::F8];
                    castle_mask = ExtraState::BLACK_KING_CASTLE | ExtraState::BLACK_QUEEN_CASTLE;
                }
                _ => unreachable!(),
            }
        }else if ty == Move::TYPE_PROMOTION{
            debug_assert_eq!(piece,Piece::player_pawn(self.state.black_turn));
            let promote = match m.promotion_piece(){
                Move::PROMOTION_QUEEN => Piece::player_queen(self.state.black_turn),
                Move::PROMOTION_KNIGHT => Piece::player_knight(self.state.black_turn),
                Move::PROMOTION_BISHOP => Piece::player_bishop(self.state.black_turn),
                Move::PROMOTION_ROOK => Piece::player_rook(self.state.black_turn),
                _ => unreachable!(),
            };

            self[piece] ^= BB::square(from);
            self[promote] ^= BB::square(to);
            self.hash ^= hasher.pieces[piece][from];
            self.hash ^= hasher.pieces[promote][to];
            self.squares[to] = Some(promote);
        }else if ty == Move::TYPE_EN_PASSANT{
            todo!()
        }else{
            self.hash ^= hasher.pieces[piece][from];
            self.hash ^= hasher.pieces[piece][to];
            self[piece] ^= BB::square(from) | BB::square(to);
            self.squares[to] = Some(piece);
            self.squares[from] = None;
        }

        castle_mask |= match to{
            Square::A1 => ExtraState::WHITE_QUEEN_CASTLE,
            Square::H1 => ExtraState::WHITE_KING_CASTLE,
            Square::A8 => ExtraState::BLACK_QUEEN_CASTLE,
            Square::H8 => ExtraState::BLACK_KING_CASTLE,
            _ => 0,
        };
        castle_mask |= match from{
            Square::A1 => ExtraState::WHITE_QUEEN_CASTLE,
            Square::H1 => ExtraState::WHITE_KING_CASTLE,
            Square::E1 => ExtraState::WHITE_KING_CASTLE | ExtraState::WHITE_QUEEN_CASTLE,
            Square::A8 => ExtraState::BLACK_QUEEN_CASTLE,
            Square::H8 => ExtraState::BLACK_KING_CASTLE,
            Square::E8 => ExtraState::BLACK_KING_CASTLE | ExtraState::BLACK_QUEEN_CASTLE,
            _ => 0,
        };

        self.state.black_turn = !self.state.black_turn;
        self.state.castle &= !castle_mask;
        self.hash ^= hasher.castle[self.state.castle as usize];

        if let Some(taken) = taken{
            self.hash ^= hasher.pieces[taken][to];
            self[taken] ^= BB::square(to);
        }

        let res = UnmakeMove{
            mov: m,
            taken,
            state,
            hash,
        };
        //self.moves.push(res);
        res
    }

    pub fn unmake_move(&mut self, mov: UnmakeMove, hasher: &Hasher) {
        //debug_assert_eq!(self.moves.pop(), Some(mov));
        self.hash ^= hasher.castle[self.state.castle as usize];
        self.hash ^= hasher.castle[mov.state.castle as usize];
        self.hash ^= hasher.black;
        self.state = mov.state;

        let from = mov.mov.from();
        let to = mov.mov.to();
        let ty = mov.mov.ty();
        let piece = self.squares[to].unwrap();

        if ty == Move::TYPE_CASTLE{
            let king = piece;
            let rook = Piece::player_rook(self.state.black_turn);
            self[king] ^= BB::square(from) | BB::square(to);
            self.squares[to] = None;
            self.squares[from] = Some(king);
            self.hash ^= hasher.pieces[king][from];
            self.hash ^= hasher.pieces[king][to];

            match to {
                Square::C1 => {
                    self.squares[Square::A1] = Some(rook);
                    self.squares[Square::D1] = None;
                    self[rook] ^= BB::A1 | BB::D1;
                    self.hash ^= hasher.pieces[rook][Square::A1];
                    self.hash ^= hasher.pieces[rook][Square::D1];
                }
                Square::G1 => {
                    self.squares[Square::H1] = Some(rook);
                    self.squares[Square::F1] = None;
                    self[rook] ^= BB::H1 | BB::F1;
                    self.hash ^= hasher.pieces[rook][Square::H1];
                    self.hash ^= hasher.pieces[rook][Square::F1];
                }
                Square::C8 => {
                    self.squares[Square::A8] = Some(rook);
                    self.squares[Square::D8] = None;
                    self[rook] ^= BB::A8 | BB::D8;
                    self.hash ^= hasher.pieces[rook][Square::A8];
                    self.hash ^= hasher.pieces[rook][Square::D8];
                }
                Square::G8 => {
                    self.squares[Square::H8] = Some(rook);
                    self.squares[Square::F8] = None;
                    self[rook] ^= BB::H8 | BB::F8;
                    self.hash ^= hasher.pieces[rook][Square::H8];
                    self.hash ^= hasher.pieces[rook][Square::F8];
                }
                _ => unreachable!(),
            }
        }else if ty == Move::TYPE_PROMOTION{
            let piece = Piece::player_pawn(self.state.black_turn);
            let promote = match mov.mov.promotion_piece(){
                Move::PROMOTION_QUEEN => Piece::player_queen(self.state.black_turn),
                Move::PROMOTION_KNIGHT => Piece::player_knight(self.state.black_turn),
                Move::PROMOTION_BISHOP => Piece::player_bishop(self.state.black_turn),
                Move::PROMOTION_ROOK => Piece::player_rook(self.state.black_turn),
                _ => unreachable!(),
            };

            self[piece] ^= BB::square(from);
            self[promote] ^= BB::square(to);
            self.hash ^= hasher.pieces[piece][from];
            self.hash ^= hasher.pieces[promote][to];
            self.squares[to] = None;
            self.squares[from] = Some(piece);
        }else if ty == Move::TYPE_EN_PASSANT{
            todo!()
        }else{
            self.hash ^= hasher.pieces[piece][from];
            self.hash ^= hasher.pieces[piece][to];
            self[piece] ^= BB::square(from) | BB::square(to);
            self.squares[to] = None;
            self.squares[from] = Some(piece);
        }

        if let Some(taken) = mov.taken{
            self[taken] |= BB::square(to);
            self.squares[to] = Some(taken);
            self.hash ^= hasher.pieces[taken][to];
        }

        self.state = mov.state;
        debug_assert_eq!(self.hash, mov.hash);
    }

    pub fn on(&self, square: Square) -> Option<Piece> {
        self.squares[square]
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
            //.field("moves", &self.moves)
            .field("hash", &self.hash)
            .field("squares", &self.squares)
            .finish()
    }
}

impl fmt::Display for Board{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in (0..8).rev(){
            write!(f,"{}: ",rank + 1)?;
            for file in 0..8{
                let s = Square::from_file_rank(file,rank);
                if let Some(x) = self.squares[s]{
                    write!(f,"{} ",x.to_char())?;
                }else{
                    write!(f,". ")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
