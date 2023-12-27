//! Functions related to the board representation.
use crate::{
    hash::HashTables, BoardArray, ExtraState, Move, Piece, PieceArray, Player, Promotion, Square,
    SquareContent, BB,
};
use std::{
    fmt::{self, Debug},
    iter::Iterator,
};

mod fen;

/// A move which has been made on the board with
/// extra information for undoing the move
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct UnmakeMove {
    /// The move taken
    pub mov: Move,
    /// Which piece was taken
    pub taken: SquareContent,
    /// The state of the previous move.
    pub state: ExtraState,
}

/// A position on the board.
#[derive(Eq, PartialEq, Clone)]
pub struct Board {
    pub pieces: PieceArray<BB>,
    pub squares: BoardArray<SquareContent>,
    pub state: ExtraState,
    pub hash: u64,
    pub hash_tables: HashTables,
}

impl Board {
    /// Returns a empty board
    pub fn empty() -> Self {
        let pieces = PieceArray::new_array([BB::EMPTY; 12]);
        let squares = BoardArray::new_array([SquareContent::Empty; 64]);
        let state = ExtraState::empty();
        let hash_tables = HashTables::new();

        Board {
            pieces,
            squares,
            state,
            hash: hash_tables.initial(),
            hash_tables,
        }
    }
}

impl Board {
    fn init_hash(&self) -> u64 {
        let mut hash = self.hash_tables.initial();
        for s in 0..64 {
            let s = Square::new(s);
            if let Some(p) = self.squares[s].to_piece() {
                hash ^= self.hash_tables.pieces()[p][s]
            }
        }
        hash ^= self.hash_tables.castle_state()[self.state.castle as usize];
        hash ^= self.hash_tables.en_passant_state()[self.state.en_passant as usize];

        if self.state.player == Player::Black {
            hash ^= self.hash_tables.turn()
        }

        hash
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

        res.pieces[Piece::WhiteKing] = BB::E1;
        res.squares[Square::E1] = SquareContent::WhiteKing;
        res.pieces[Piece::WhiteQueen] = BB::D1;
        res.squares[Square::D1] = SquareContent::WhiteQueen;
        res.pieces[Piece::WhiteBishop] = BB::C1 | BB::F1;
        res.squares[Square::C1] = SquareContent::WhiteBishop;
        res.squares[Square::F1] = SquareContent::WhiteBishop;
        res.pieces[Piece::WhiteKnight] = BB::B1 | BB::G1;
        res.squares[Square::B1] = SquareContent::WhiteKnight;
        res.squares[Square::G1] = SquareContent::WhiteKnight;
        res.pieces[Piece::WhiteRook] = BB::A1 | BB::H1;
        res.squares[Square::A1] = SquareContent::WhiteRook;
        res.squares[Square::H1] = SquareContent::WhiteRook;
        res.pieces[Piece::WhitePawn] = BB::RANK_2;

        let mut f = 0;
        while f < 8 {
            res.squares[Square::from_file_rank(f, 1)] = SquareContent::WhitePawn;
            res.squares[Square::from_file_rank(f, 6)] = SquareContent::BlackPawn;
            f += 1;
        }

        res.pieces[Piece::BlackKing] = BB::E8;
        res.squares[Square::E8] = SquareContent::BlackKing;
        res.pieces[Piece::BlackQueen] = BB::D8;
        res.squares[Square::D8] = SquareContent::BlackQueen;
        res.pieces[Piece::BlackBishop] = BB::C8 | BB::F8;
        res.squares[Square::C8] = SquareContent::BlackBishop;
        res.squares[Square::F8] = SquareContent::BlackBishop;
        res.pieces[Piece::BlackKnight] = BB::B8 | BB::G8;
        res.squares[Square::B8] = SquareContent::BlackKnight;
        res.squares[Square::G8] = SquareContent::BlackKnight;
        res.pieces[Piece::BlackRook] = BB::A8 | BB::H8;
        res.squares[Square::A8] = SquareContent::BlackRook;
        res.squares[Square::H8] = SquareContent::BlackRook;
        res.pieces[Piece::BlackPawn] = BB::RANK_7;

        res.hash = res.init_hash();

        res
    }

    pub fn copy_position(&mut self, b: Board) {
        self.squares = b.squares;
        self.pieces = b.pieces;
        self.state = b.state;
        self.hash_tables = b.hash_tables;
        self.hash = b.hash;
    }

    pub fn is_equal(&self, other: &Self) -> bool {
        if self.hash != other.hash {
            return false;
        }

        for p in Piece::all() {
            if self.pieces[p] != other.pieces[p] {
                return false;
            }
        }

        if self.state != other.state {
            return false;
        }

        true
    }

    /// Checks whether the given position is valid.
    pub fn is_valid(&self) -> bool {
        let mut res = true;

        if self.pieces[Piece::WhiteKing].count() != 1 {
            res = false;
            eprintln!(
                "Wrong number of white kings\n{:?}",
                self.pieces[Piece::WhiteKing]
            );
        }
        if self.pieces[Piece::BlackKing].count() != 1 {
            res = false;
            eprintln!(
                "Wrong number of black kings\n{:?}",
                self.pieces[Piece::BlackKing]
            );
        }
        for pa in Piece::all() {
            for pb in Piece::all() {
                if pa == pb {
                    break;
                }
                if !(self.pieces[pa] & self.pieces[pb]).none() {
                    eprintln!(
                        "Overlap in bitboards: {:?} {:?}\n{:?}{:?}",
                        pa, pb, self.pieces[pa], self.pieces[pb]
                    );
                    res = false;
                }
            }
        }

        for s in 0..64 {
            let s = Square::new(s);
            if let Some(x) = self.squares[s].to_piece() {
                if !(self.pieces[x] & BB::square(s)).any() {
                    eprintln!("mailbox-bitboard mismatch, Square {} should contain {:?} but bitboard does not:\n{:?}",s,x,self.pieces[x]);
                    res = false;
                }
            } else {
                let sb = BB::square(s);
                for p in Piece::all() {
                    if !(self.pieces[p] & sb).none() {
                        eprintln!("mailbox-bitboard mismatch, Square {} should be empty but contains {:?} on bitboard\n{:?}",s,p,self.pieces[p]);
                        res = false;
                    }
                }
            }
        }

        let should_hash = self.init_hash();
        if self.hash != should_hash {
            eprintln!(
                "hash corrupted expected: {} found {}",
                self.init_hash(),
                self.hash
            );

            for p in Piece::all() {
                for s in 0..64 {
                    let s = Square::new(s);

                    if self.hash ^ self.hash_tables.pieces()[p][s] == should_hash {
                        eprintln!("would be fixed after {p:?} on {s}")
                    }
                }
            }

            for s in 0..16 {
                if self.hash ^ self.hash_tables.castle_state()[s] == should_hash {
                    eprintln!("would be fixed after castle_state {s}")
                }
            }

            for s in 0..9 {
                if self.hash ^ self.hash_tables.en_passant_state()[s] == should_hash {
                    eprintln!("would be fixed after en_passant_state {s}")
                }
            }

            if self.hash ^ self.hash_tables.turn() == should_hash {
                eprintln!("would be fixed after turn")
            }

            res = false
        }

        res
    }

    /// Make a move on the board.
    #[inline]
    pub fn make_move(&mut self, m: Move) -> UnmakeMove {
        assert_ne!(m, Move::INVALID);

        self.hash ^= self.hash_tables.turn();

        if m == Move::NULL {
            let res = UnmakeMove {
                mov: m,
                state: self.state,
                taken: SquareContent::Empty,
            };
            self.state.player = !self.state.player;
            return res;
        }

        let old_state = self.state;

        let from = m.from();
        let to = m.to();
        let ty = m.ty();

        // do the stuff which is always required.
        let piece = std::mem::replace(&mut self.squares[from], SquareContent::Empty);
        let Some(piece) = piece.to_piece() else {
            panic!("tried to move piece which does not exist. On square `{from}`\n{self}\n{self:?}")
        };

        let mut taken = std::mem::replace(&mut self.squares[to], piece.into());
        self.pieces[piece] ^= BB::square(from) | BB::square(to);
        self.hash ^= self.hash_tables.pieces()[piece][from];
        self.hash ^= self.hash_tables.pieces()[piece][to];

        let mut reversible = taken == SquareContent::Empty;

        assert!(
            !matches!(taken, SquareContent::WhiteKing | SquareContent::BlackKing),
            "tried to take a king {taken:?} on {to} with {piece:?}\n{self:?}",
        );

        self.hash ^= self.hash_tables.en_passant_state()[self.state.en_passant as usize];
        self.state.en_passant = ExtraState::INVALID_ENPASSANT;
        self.hash ^= self.hash_tables.en_passant_state()[self.state.en_passant as usize];

        if ty == Move::TYPE_NORMAL {
            if let Some(taken) = taken.to_piece() {
                self.pieces[taken] &= !BB::square(to);
                self.hash ^= self.hash_tables.pieces()[taken][to];
            }

            if m.is_double_move() {
                self.hash ^= self.hash_tables.en_passant_state()[self.state.en_passant as usize];
                self.state.en_passant = from.file();
                self.hash ^= self.hash_tables.en_passant_state()[self.state.en_passant as usize];
            };

            let castle_mask = match to {
                Square::A1 => ExtraState::WHITE_QUEEN_CASTLE,
                Square::H1 => ExtraState::WHITE_KING_CASTLE,
                Square::A8 => ExtraState::BLACK_QUEEN_CASTLE,
                Square::H8 => ExtraState::BLACK_KING_CASTLE,
                _ => 0,
            } | match from {
                Square::A1 => ExtraState::WHITE_QUEEN_CASTLE,
                Square::H1 => ExtraState::WHITE_KING_CASTLE,
                Square::E1 => ExtraState::WHITE_KING_CASTLE | ExtraState::WHITE_QUEEN_CASTLE,
                Square::A8 => ExtraState::BLACK_QUEEN_CASTLE,
                Square::H8 => ExtraState::BLACK_KING_CASTLE,
                Square::E8 => ExtraState::BLACK_KING_CASTLE | ExtraState::BLACK_QUEEN_CASTLE,
                _ => 0,
            };

            // TODO: Possibly make branchless.
            reversible = reversible
                && castle_mask & self.state.castle == 0
                && !matches!(piece, Piece::WhitePawn | Piece::BlackPawn);

            self.hash ^= self.hash_tables.castle_state()[self.state.castle as usize];
            self.state.castle &= !castle_mask;
            self.hash ^= self.hash_tables.castle_state()[self.state.castle as usize];
        } else if ty == Move::TYPE_CASTLE {
            reversible = false;
            match to {
                Square::C1 => {
                    self.squares[Square::A1] = SquareContent::Empty;
                    self.squares[Square::D1] = SquareContent::WhiteRook;
                    self.pieces[Piece::WhiteRook] ^= BB::A1 | BB::D1;
                    self.hash ^= self.hash_tables.castle_state()[self.state.castle as usize];
                    self.state.castle &=
                        !(ExtraState::WHITE_KING_CASTLE | ExtraState::WHITE_QUEEN_CASTLE);
                    self.hash ^= self.hash_tables.castle_state()[self.state.castle as usize];
                    self.hash ^= self.hash_tables.pieces()[Piece::WhiteRook][Square::A1];
                    self.hash ^= self.hash_tables.pieces()[Piece::WhiteRook][Square::D1];
                }
                Square::G1 => {
                    self.squares[Square::H1] = SquareContent::Empty;
                    self.squares[Square::F1] = SquareContent::WhiteRook;
                    self.pieces[Piece::WhiteRook] ^= BB::H1 | BB::F1;
                    self.hash ^= self.hash_tables.castle_state()[self.state.castle as usize];
                    self.state.castle &=
                        !(ExtraState::WHITE_KING_CASTLE | ExtraState::WHITE_QUEEN_CASTLE);
                    self.hash ^= self.hash_tables.castle_state()[self.state.castle as usize];
                    self.hash ^= self.hash_tables.pieces()[Piece::WhiteRook][Square::H1];
                    self.hash ^= self.hash_tables.pieces()[Piece::WhiteRook][Square::F1];
                }
                Square::C8 => {
                    self.squares[Square::A8] = SquareContent::Empty;
                    self.squares[Square::D8] = SquareContent::BlackRook;
                    self.pieces[Piece::BlackRook] ^= BB::A8 | BB::D8;
                    self.hash ^= self.hash_tables.castle_state()[self.state.castle as usize];
                    self.state.castle &=
                        !(ExtraState::BLACK_KING_CASTLE | ExtraState::BLACK_QUEEN_CASTLE);
                    self.hash ^= self.hash_tables.castle_state()[self.state.castle as usize];
                    self.hash ^= self.hash_tables.pieces()[Piece::BlackRook][Square::A8];
                    self.hash ^= self.hash_tables.pieces()[Piece::BlackRook][Square::D8];
                }
                Square::G8 => {
                    self.squares[Square::H8] = SquareContent::Empty;
                    self.squares[Square::F8] = SquareContent::BlackRook;
                    self.pieces[Piece::BlackRook] ^= BB::H8 | BB::F8;
                    self.hash ^= self.hash_tables.castle_state()[self.state.castle as usize];
                    self.state.castle &=
                        !(ExtraState::BLACK_KING_CASTLE | ExtraState::BLACK_QUEEN_CASTLE);
                    self.hash ^= self.hash_tables.castle_state()[self.state.castle as usize];
                    self.hash ^= self.hash_tables.pieces()[Piece::BlackRook][Square::H8];
                    self.hash ^= self.hash_tables.pieces()[Piece::BlackRook][Square::F8];
                }
                _ => unreachable!(),
            }
        } else if ty == Move::TYPE_PROMOTION {
            reversible = false;
            debug_assert_eq!(piece, Piece::player_pawn(self.state.player));

            if let Some(taken) = taken.to_piece() {
                self.pieces[taken] &= !BB::square(to);
                self.hash ^= self.hash_tables.pieces()[taken][to];
            }

            let promote = match m.promotion_piece() {
                Promotion::Queen => Piece::player_queen(self.state.player),
                Promotion::Knight => Piece::player_knight(self.state.player),
                Promotion::Bishop => Piece::player_bishop(self.state.player),
                Promotion::Rook => Piece::player_rook(self.state.player),
            };

            self.pieces[piece] &= !BB::square(to);
            self.pieces[promote] |= BB::square(to);
            self.squares[to] = promote.into();

            self.hash ^= self.hash_tables.pieces()[piece][to];
            self.hash ^= self.hash_tables.pieces()[promote][to];

            self.hash ^= self.hash_tables.castle_state()[self.state.castle as usize];
            self.state.castle &= !match to {
                Square::A1 => ExtraState::WHITE_QUEEN_CASTLE,
                Square::H1 => ExtraState::WHITE_KING_CASTLE,
                Square::A8 => ExtraState::BLACK_QUEEN_CASTLE,
                Square::H8 => ExtraState::BLACK_KING_CASTLE,
                _ => 0,
            };
            self.hash ^= self.hash_tables.castle_state()[self.state.castle as usize];
        } else if ty == Move::TYPE_EN_PASSANT {
            reversible = false;
            let (taken_sq, taken_piece) = match self.state.player {
                Player::White => (to - 8i8, Piece::BlackPawn),
                Player::Black => (to + 8i8, Piece::WhitePawn),
            };

            self.pieces[taken_piece] &= !BB::square(taken_sq);
            self.squares[taken_sq] = SquareContent::Empty;
            self.hash ^= self.hash_tables.pieces()[taken_piece][taken_sq];

            taken = taken_piece.into();
        }

        self.state.player = !self.state.player;

        if reversible {
            self.state.move_clock += 1;
        } else {
            self.state.move_clock = 0;
        }

        UnmakeMove {
            mov: m,
            taken,
            state: old_state,
        }
    }

    /// Undo a move
    #[inline]
    pub fn unmake_move(&mut self, mov: UnmakeMove) {
        self.hash ^= self.hash_tables.turn();

        if mov.mov == Move::NULL {
            self.state.player = !self.state.player;
            return;
        }

        let from = mov.mov.from();
        let to = mov.mov.to();
        let ty = mov.mov.ty();

        let piece = std::mem::replace(&mut self.squares[to], mov.taken);
        self.squares[from] = piece;
        let Some(piece) = piece.to_piece() else {
            panic!("tried to undo move to a square which is empty");
        };
        self.pieces[piece] ^= BB::square(from) | BB::square(to);
        self.hash ^= self.hash_tables.pieces()[piece][from];
        self.hash ^= self.hash_tables.pieces()[piece][to];

        self.hash ^= self.hash_tables.castle_state()[self.state.castle as usize];
        self.hash ^= self.hash_tables.castle_state()[mov.state.castle as usize];

        self.hash ^= self.hash_tables.en_passant_state()[self.state.en_passant as usize];
        self.hash ^= self.hash_tables.en_passant_state()[mov.state.en_passant as usize];

        self.state = mov.state;

        if ty == Move::TYPE_NORMAL {
            if let Some(taken) = mov.taken.to_piece() {
                self.hash ^= self.hash_tables.pieces()[taken][to];
                self.pieces[taken] |= BB::square(to);
            }
        } else if ty == Move::TYPE_CASTLE {
            match to {
                Square::C1 => {
                    self.squares[Square::D1] = SquareContent::Empty;
                    self.squares[Square::A1] = SquareContent::WhiteRook;
                    self.pieces[Piece::WhiteRook] ^= BB::D1 | BB::A1;
                    self.hash ^= self.hash_tables.pieces()[Piece::WhiteRook][Square::A1];
                    self.hash ^= self.hash_tables.pieces()[Piece::WhiteRook][Square::D1];
                }
                Square::G1 => {
                    self.squares[Square::F1] = SquareContent::Empty;
                    self.squares[Square::H1] = SquareContent::WhiteRook;
                    self.pieces[Piece::WhiteRook] ^= BB::F1 | BB::H1;
                    self.hash ^= self.hash_tables.pieces()[Piece::WhiteRook][Square::H1];
                    self.hash ^= self.hash_tables.pieces()[Piece::WhiteRook][Square::F1];
                }
                Square::C8 => {
                    self.squares[Square::D8] = SquareContent::Empty;
                    self.squares[Square::A8] = SquareContent::BlackRook;
                    self.pieces[Piece::BlackRook] ^= BB::D8 | BB::A8;
                    self.hash ^= self.hash_tables.pieces()[Piece::BlackRook][Square::A8];
                    self.hash ^= self.hash_tables.pieces()[Piece::BlackRook][Square::D8];
                }
                Square::G8 => {
                    self.squares[Square::F8] = SquareContent::Empty;
                    self.squares[Square::H8] = SquareContent::BlackRook;
                    self.pieces[Piece::BlackRook] ^= BB::F8 | BB::H8;
                    self.hash ^= self.hash_tables.pieces()[Piece::BlackRook][Square::H8];
                    self.hash ^= self.hash_tables.pieces()[Piece::BlackRook][Square::F8];
                }
                _ => unreachable!(),
            }
        } else if ty == Move::TYPE_PROMOTION {
            if let Some(taken) = mov.taken.to_piece() {
                self.pieces[taken] |= BB::square(to);
                self.hash ^= self.hash_tables.pieces()[taken][to];
            }

            let pawn = Piece::player_pawn(self.state.player);

            self.pieces[piece] &= !BB::square(from);
            self.pieces[pawn] |= BB::square(from);
            self.squares[from] = pawn.into();

            self.hash ^= self.hash_tables.pieces()[piece][from];
            self.hash ^= self.hash_tables.pieces()[pawn][from];
        } else if ty == Move::TYPE_EN_PASSANT {
            let m: i8 = match self.state.player {
                Player::Black => 8,
                Player::White => -8,
            };
            let taken_sq = to + m;
            self.squares[to] = SquareContent::Empty;
            self.squares[taken_sq] = mov.taken;
            let piece = mov.taken.to_piece().unwrap();
            self.pieces[piece] |= BB::square(taken_sq);
            self.hash ^= self.hash_tables.pieces()[piece][taken_sq];
        }

        self.state = mov.state;
    }

    #[inline]
    pub fn on(&self, square: Square) -> SquareContent {
        self.squares[square]
    }
}

impl Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Board")
            .field("white_king", &self.pieces[Piece::WhiteKing])
            .field("white_queen", &self.pieces[Piece::WhiteQueen])
            .field("white_bishop", &self.pieces[Piece::WhiteBishop])
            .field("white_knight", &self.pieces[Piece::WhiteKnight])
            .field("white_rook", &self.pieces[Piece::WhiteRook])
            .field("white_pawn", &self.pieces[Piece::WhitePawn])
            .field("black_king", &self.pieces[Piece::BlackKing])
            .field("black_queen", &self.pieces[Piece::BlackQueen])
            .field("black_bishop", &self.pieces[Piece::BlackBishop])
            .field("black_knight", &self.pieces[Piece::BlackKnight])
            .field("black_rook", &self.pieces[Piece::BlackRook])
            .field("black_pawn", &self.pieces[Piece::BlackPawn])
            .field("state", &self.state)
            .field("squares", &self.squares)
            .field("hash", &self.hash)
            .finish()
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in (0..8).rev() {
            write!(f, "{}: ", rank + 1)?;
            for file in 0..8 {
                let s = Square::from_file_rank(file, rank);
                write!(f, "{} ", self.squares[s])?;
            }
            writeln!(f)?;
        }
        write!(f, "   ")?;
        for file in 0..8 {
            write!(f, "{} ", (b'a' + file) as char)?;
        }
        writeln!(f)
    }
}
