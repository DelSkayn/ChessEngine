use crate::{
    hash::Hasher,
    util::{BoardArray, PieceArray},
    ExtraState, Move, Piece, Player, Square, BB,
};

use std::{
    fmt::{self, Debug},
    iter::Iterator,
    ops::{Index, IndexMut},
};

/// A move which has been made on the board with
/// extra information regarding undoing the move
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct UnmakeMove {
    pub mov: Move,
    taken: Option<Piece>,
    state: ExtraState,
    hash: u64,
}

/// A position on the board.
#[derive(Eq, PartialEq, Clone)]
pub struct Board {
    pub pieces: PieceArray<BB>,
    pub state: ExtraState,
    pub squares: BoardArray<Option<Piece>>,
    pub hash: u64,
}

impl Board {
    /// Returns a empty board
    ///
    /// Board has not initialized the hash value
    pub const fn empty() -> Self {
        Board {
            pieces: PieceArray::new_array([BB::EMPTY; 12]),
            squares: BoardArray::new_array([None; 64]),
            state: ExtraState::empty(),
            //moves: Vec::new(),
            hash: 0,
        }
    }

    /// Returns a board in the start position
    ///
    /// Board has not initialized the hash value
    pub fn start_position() -> Self {
        let mut res = Board::empty();
        res.state.castle = ExtraState::BLACK_KING_CASTLE
            | ExtraState::BLACK_QUEEN_CASTLE
            | ExtraState::WHITE_KING_CASTLE
            | ExtraState::WHITE_QUEEN_CASTLE;
        res.state.player = Player::White;

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
        for f in 0..8 {
            res.squares[Square::from_file_rank(f, 1)] = Some(Piece::WhitePawn);
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
        for f in 0..8 {
            res.squares[Square::from_file_rank(f, 6)] = Some(Piece::BlackPawn);
        }

        res
    }

    pub fn is_equal(&self, other: &Self) -> bool {
        if self.hash != other.hash {
            println!("hash not equal");
            return false;
        }

        for p in Piece::WhiteKing.to(Piece::BlackPawn) {
            if self[p] != other[p] {
                println!("{:?}:{:?} != {:?}", p, self[p], other[p]);
                return false;
            }
        }

        if self.state != other.state {
            println!("state not equal");
            return false;
        }

        if self.squares != other.squares {
            println!("squaers not equal");
            return false;
        }

        true
    }

    /// Checks whether the given position is valid.
    pub fn is_valid(&self) -> bool {
        let mut res = true;

        if self[Piece::WhiteKing].count() != 1 {
            res = false;
            eprintln!("Wrong number of white kings\n{:?}", self[Piece::WhiteKing]);
        }
        if self[Piece::BlackKing].count() != 1 {
            res = false;
            eprintln!("Wrong number of black kings\n{:?}", self[Piece::BlackKing]);
        }
        for pa in Piece::WhiteKing.to(Piece::BlackPawn) {
            for pb in Piece::WhiteKing.to(pa) {
                if pa == pb {
                    continue;
                }
                if !(self[pa] & self[pb]).none() {
                    eprintln!(
                        "Overlap in bitboards: {:?} {:?}\n{:?}{:?}",
                        pa, pb, self[pa], self[pb]
                    );
                    res = false;
                }
            }
        }

        for s in 0..64 {
            let s = Square::new(s);
            if let Some(x) = self.squares[s] {
                if !(self[x] & BB::square(s)).any() {
                    eprintln!("mailbox-bitboard mismatch, Square {} should contain {:?} but bitboard does not:\n{:?}",s,x,self[x]);
                    res = false;
                }
            } else {
                let sb = BB::square(s);
                for p in Piece::WhiteKing.to(Piece::BlackPawn) {
                    if !(self[p] & sb).none() {
                        eprintln!("mailbox-bitboard mismatch, Square {} should be empty but contains {:?} on bitboard\n{:?}",s,p,self[p]);
                        res = false;
                    }
                }
            }
        }

        res
    }

    pub fn calc_hash(&mut self, hasher: &Hasher) {
        self.hash = hasher.build(&self.pieces, self.state);
    }

    pub fn flip(mut self) -> Self {
        for p in Piece::WhiteKing.to(Piece::WhitePawn) {
            let cur = p;
            let other = p.flip(true);
            self.pieces[cur].flip();
            self.pieces[other].flip();
            let o = self.pieces[other];
            self.pieces[cur] ^= o;
            let c = self.pieces[cur];
            self.pieces[other] ^= c;
            let o = self.pieces[other];
            self.pieces[cur] ^= o;
        }
        self.state = self.state.flip();
        return self;
    }

    /// Make a move on the board.
    pub fn make_move(&mut self, m: Move, hasher: &Hasher) -> UnmakeMove {
        //debug_assert!(self.hash != 0);
        let state = self.state;
        let hash = self.hash;
        self.hash ^= hasher.black;
        self.hash ^= hasher.castle[self.state.castle as usize];

        self.state.en_passant = ExtraState::INVALID_ENPASSANT;

        let from = m.from();
        let to = m.to();
        let ty = m.ty();
        if self.squares[from].is_none() {
            println!("{:?}", self);
            println!("{}", self);
        }
        let piece = self.squares[from].unwrap();
        let mut taken = self.squares[to];

        self.squares[from] = None;
        let mut castle_mask = 0;

        if ty == Move::TYPE_NORMAL {
            self.hash ^= hasher.pieces[piece][from];
            self.hash ^= hasher.pieces[piece][to];
            self[piece] ^= BB::square(from) | BB::square(to);
            self.squares[to] = Some(piece);
            self.squares[from] = None;

            if let Some(taken) = taken {
                self.hash ^= hasher.pieces[taken][to];
                self[taken] ^= BB::square(to);
            }

            if m.is_double_move() {
                self.state.en_passant = from.file();
            }
        } else if ty == Move::TYPE_CASTLE {
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
        } else if ty == Move::TYPE_PROMOTION {
            debug_assert_eq!(piece, Piece::player_pawn(self.state.player));
            let promote = match m.promotion_piece() {
                Move::PROMOTION_QUEEN => Piece::player_queen(self.state.player),
                Move::PROMOTION_KNIGHT => Piece::player_knight(self.state.player),
                Move::PROMOTION_BISHOP => Piece::player_bishop(self.state.player),
                Move::PROMOTION_ROOK => Piece::player_rook(self.state.player),
                _ => unreachable!(),
            };

            self[piece] ^= BB::square(from);
            self[promote] ^= BB::square(to);
            self.hash ^= hasher.pieces[piece][from];
            self.hash ^= hasher.pieces[promote][to];
            self.squares[to] = Some(promote);

            if let Some(taken) = taken {
                self.hash ^= hasher.pieces[taken][to];
                self[taken] ^= BB::square(to);
            }
        } else if ty == Move::TYPE_EN_PASSANT {
            let (taken_sq, taken_piece) = match self.state.player {
                Player::White => (to - 8i8, Piece::BlackPawn),
                Player::Black => (to + 8i8, Piece::WhitePawn),
            };
            self[piece] ^= BB::square(from);
            self[piece] ^= BB::square(to);
            self[taken_piece] ^= BB::square(taken_sq);

            self.hash ^= hasher.pieces[piece][from];
            self.hash ^= hasher.pieces[piece][to];
            self.hash ^= hasher.pieces[taken_piece][taken_sq];
            self.squares[to] = Some(piece);
            self.squares[from] = None;
            self.squares[taken_sq] = None;
            taken = Some(taken_piece);
        }

        castle_mask |= match to {
            Square::A1 => ExtraState::WHITE_QUEEN_CASTLE,
            Square::H1 => ExtraState::WHITE_KING_CASTLE,
            Square::A8 => ExtraState::BLACK_QUEEN_CASTLE,
            Square::H8 => ExtraState::BLACK_KING_CASTLE,
            _ => 0,
        };
        castle_mask |= match from {
            Square::A1 => ExtraState::WHITE_QUEEN_CASTLE,
            Square::H1 => ExtraState::WHITE_KING_CASTLE,
            Square::E1 => ExtraState::WHITE_KING_CASTLE | ExtraState::WHITE_QUEEN_CASTLE,
            Square::A8 => ExtraState::BLACK_QUEEN_CASTLE,
            Square::H8 => ExtraState::BLACK_KING_CASTLE,
            Square::E8 => ExtraState::BLACK_KING_CASTLE | ExtraState::BLACK_QUEEN_CASTLE,
            _ => 0,
        };

        self.state.player = self.state.player.flip();
        self.state.castle &= !castle_mask;
        self.hash ^= hasher.castle[self.state.castle as usize];

        let res = UnmakeMove {
            mov: m,
            taken,
            state,
            hash,
        };
        //self.moves.push(res);
        res
    }

    /// Undo a move
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

        if ty == Move::TYPE_NORMAL {
            self.hash ^= hasher.pieces[piece][from];
            self.hash ^= hasher.pieces[piece][to];
            self[piece] ^= BB::square(from) | BB::square(to);
            self.squares[to] = None;
            self.squares[from] = Some(piece);

            if let Some(taken) = mov.taken {
                self[taken] |= BB::square(to);
                self.squares[to] = Some(taken);
                self.hash ^= hasher.pieces[taken][to];
            }
        } else if ty == Move::TYPE_CASTLE {
            let king = piece;
            let rook = Piece::player_rook(self.state.player);
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
        } else if ty == Move::TYPE_PROMOTION {
            let piece = Piece::player_pawn(self.state.player);
            let promote = match mov.mov.promotion_piece() {
                Move::PROMOTION_QUEEN => Piece::player_queen(self.state.player),
                Move::PROMOTION_KNIGHT => Piece::player_knight(self.state.player),
                Move::PROMOTION_BISHOP => Piece::player_bishop(self.state.player),
                Move::PROMOTION_ROOK => Piece::player_rook(self.state.player),
                _ => unreachable!(),
            };

            self[piece] ^= BB::square(from);
            self[promote] ^= BB::square(to);
            self.hash ^= hasher.pieces[piece][from];
            self.hash ^= hasher.pieces[promote][to];
            self.squares[to] = None;
            self.squares[from] = Some(piece);

            if let Some(taken) = mov.taken {
                self[taken] |= BB::square(to);
                self.squares[to] = Some(taken);
                self.hash ^= hasher.pieces[taken][to];
            }
        } else if ty == Move::TYPE_EN_PASSANT {
            let m: i8 = match self.state.player {
                Player::Black => 8,
                Player::White => -8,
            };
            let taken_sq = to + m;
            let taken = mov.taken.unwrap();
            self[piece] ^= BB::square(from) | BB::square(to);
            self[taken] |= BB::square(taken_sq);

            self.hash ^= hasher.pieces[piece][from];
            self.hash ^= hasher.pieces[piece][to];
            self.hash ^= hasher.pieces[taken][taken_sq];
            self.squares[to] = None;
            self.squares[from] = Some(piece);
            self.squares[taken_sq] = Some(taken);
        }

        self.state = mov.state;
        debug_assert_eq!(self.hash, mov.hash);
    }

    #[inline(always)]
    pub fn on(&self, square: Square) -> Option<Piece> {
        self.squares[square]
    }
}

impl Index<Piece> for Board {
    type Output = BB;
    #[inline(always)]
    fn index(&self, index: Piece) -> &Self::Output {
        &self.pieces[index]
    }
}

impl IndexMut<Piece> for Board {
    #[inline(always)]
    fn index_mut(&mut self, index: Piece) -> &mut Self::Output {
        &mut self.pieces[index]
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

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in (0..8).rev() {
            write!(f, "{}: ", rank + 1)?;
            for file in 0..8 {
                let s = Square::from_file_rank(file, rank);
                if let Some(x) = self.squares[s] {
                    write!(f, "{} ", x.to_char())?;
                } else {
                    write!(f, ". ")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
