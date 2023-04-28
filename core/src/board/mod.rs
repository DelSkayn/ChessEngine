//! Functions related to the board representation.

use crate::{
    bb::BB,
    mov::Promotion,
    util::{BoardArray, PieceArray},
    ExtraState, Move, Piece, Player, Square,
};
use std::{
    fmt::{self, Debug},
    iter::Iterator,
};

mod chain;
mod fen;
pub use chain::{EndChain, HashChain, MoveChain};

/// A move which has been made on the board with
/// extra information regarding undoing the move
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct UnmakeMove {
    pub mov: Move,
    taken: Option<Piece>,
    state: ExtraState,
}

/// A position on the board.
#[derive(Eq, PartialEq, Clone)]
pub struct Board<C: MoveChain = EndChain> {
    pub pieces: PieceArray<BB>,
    pub state: ExtraState,
    pub squares: BoardArray<Option<Piece>>,
    pub chain: C,
}

impl Board<EndChain> {
    /// Returns a empty board
    ///
    /// Board has not initialized the hash value
    pub fn empty() -> Self {
        let pieces = PieceArray::new_array([BB::EMPTY; 12]);
        let squares = BoardArray::new_array([None; 64]);
        let state = ExtraState::empty();

        Board {
            pieces,
            squares,
            state,
            chain: EndChain,
        }
    }
}

impl<C> Board<C>
where
    C: MoveChain,
{
    /// Returns a board in the start position
    ///
    /// Board has not initialized the hash value
    pub fn start_position(mut chain: C) -> Self {
        let mut res = Board::empty();
        res.state.castle = ExtraState::BLACK_KING_CASTLE
            | ExtraState::BLACK_QUEEN_CASTLE
            | ExtraState::WHITE_KING_CASTLE
            | ExtraState::WHITE_QUEEN_CASTLE;
        res.state.player = Player::White;

        res.pieces[Piece::WhiteKing] = BB::E1;
        res.squares[Square::E1] = Some(Piece::WhiteKing);
        res.pieces[Piece::WhiteQueen] = BB::D1;
        res.squares[Square::D1] = Some(Piece::WhiteQueen);
        res.pieces[Piece::WhiteBishop] = BB::C1 | BB::F1;
        res.squares[Square::C1] = Some(Piece::WhiteBishop);
        res.squares[Square::F1] = Some(Piece::WhiteBishop);
        res.pieces[Piece::WhiteKnight] = BB::B1 | BB::G1;
        res.squares[Square::B1] = Some(Piece::WhiteKnight);
        res.squares[Square::G1] = Some(Piece::WhiteKnight);
        res.pieces[Piece::WhiteRook] = BB::A1 | BB::H1;
        res.squares[Square::A1] = Some(Piece::WhiteRook);
        res.squares[Square::H1] = Some(Piece::WhiteRook);
        res.pieces[Piece::WhitePawn] = BB::RANK_2;
        for f in 0..8 {
            res.squares[Square::from_file_rank(f, 1)] = Some(Piece::WhitePawn);
        }

        res.pieces[Piece::BlackKing] = BB::E8;
        res.squares[Square::E8] = Some(Piece::BlackKing);
        res.pieces[Piece::BlackQueen] = BB::D8;
        res.squares[Square::D8] = Some(Piece::BlackQueen);
        res.pieces[Piece::BlackBishop] = BB::C8 | BB::F8;
        res.squares[Square::C8] = Some(Piece::BlackBishop);
        res.squares[Square::F8] = Some(Piece::BlackBishop);
        res.pieces[Piece::BlackKnight] = BB::B8 | BB::G8;
        res.squares[Square::B8] = Some(Piece::BlackKnight);
        res.squares[Square::G8] = Some(Piece::BlackKnight);
        res.pieces[Piece::BlackRook] = BB::A8 | BB::H8;
        res.squares[Square::A8] = Some(Piece::BlackRook);
        res.squares[Square::H8] = Some(Piece::BlackRook);
        res.pieces[Piece::BlackPawn] = BB::RANK_7;
        for f in 0..8 {
            res.squares[Square::from_file_rank(f, 6)] = Some(Piece::BlackPawn);
        }

        chain.position(&res.pieces, res.state);

        Board {
            pieces: res.pieces,
            squares: res.squares,
            state: res.state,
            chain,
        }
    }

    pub fn copy_position<H: MoveChain>(&mut self, b: &Board<H>) {
        self.squares = b.squares;
        self.pieces = b.pieces;
        self.state = b.state;
        self.chain.position(&self.pieces, self.state);
    }

    pub fn is_equal(&self, other: &Self) -> bool {
        for p in Piece::WhiteKing.to(Piece::BlackPawn) {
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
        for pa in Piece::WhiteKing.to(Piece::BlackPawn) {
            for pb in Piece::WhiteKing.to(pa) {
                if pa == pb {
                    continue;
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
            if let Some(x) = self.squares[s] {
                if !(self.pieces[x] & BB::square(s)).any() {
                    eprintln!("mailbox-bitboard mismatch, Square {} should contain {:?} but bitboard does not:\n{:?}",s,x,self.pieces[x]);
                    res = false;
                }
            } else {
                let sb = BB::square(s);
                for p in Piece::WhiteKing.to(Piece::BlackPawn) {
                    if !(self.pieces[p] & sb).none() {
                        eprintln!("mailbox-bitboard mismatch, Square {} should be empty but contains {:?} on bitboard\n{:?}",s,p,self.pieces[p]);
                        res = false;
                    }
                }
            }
        }

        res
    }

    /*
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
    */

    #[inline]
    fn move_piece(&mut self, piece: Piece, from: Square, to: Square) {
        self.squares[to] = Some(piece);
        self.squares[from] = None;
        self.pieces[piece] ^= BB::square(from) | BB::square(to);
        self.chain.move_piece(piece, from, to);
    }

    #[inline]
    fn take_piece(&mut self, taken: Piece, square: Square) {
        self.pieces[taken] ^= BB::square(square);
        self.squares[square] = None;
        self.chain.take_piece(taken, square);
    }

    #[inline]
    fn untake_piece(&mut self, taken: Piece, square: Square) {
        self.pieces[taken] ^= BB::square(square);
        self.squares[square] = Some(taken);
        self.chain.untake_piece(taken, square);
    }

    #[inline]
    fn promote_piece(&mut self, piece: Piece, promote: Piece, from: Square, to: Square) {
        self.pieces[piece] ^= BB::square(from);
        self.pieces[promote] ^= BB::square(to);
        self.squares[from] = None;
        self.squares[to] = Some(promote);
        self.chain.promote_piece(piece, promote, from, to);
    }

    fn unpromote_piece(&mut self, piece: Piece, promote: Piece, from: Square, to: Square) {
        self.pieces[piece] ^= BB::square(from);
        self.pieces[promote] ^= BB::square(to);
        self.squares[to] = None;
        self.squares[from] = Some(piece);
        self.chain.unpromote_piece(piece, promote, from, to);
    }

    /// Make a move on the board.
    pub fn make_move(&mut self, m: Move) -> UnmakeMove {
        assert_ne!(m, Move::INVALID);
        //debug_assert!(self.hash != 0);
        let state = self.state;

        self.chain.move_start(self.state);

        self.state.en_passant = ExtraState::INVALID_ENPASSANT;

        let from = m.from();
        let to = m.to();
        let ty = m.ty();
        /*if self.squares[from].is_none() {
            println!("{:?}", self);
            println!("{}", self);
        }*/
        let piece = self.squares[from]
            .ok_or_else(|| format!("invalid lookup: {}\n{:?}", from, self.squares))
            .unwrap();
        let mut taken = self.squares[to];
        assert_ne!(taken, Some(Piece::WhiteKing), "{}\n{:?}", m, self.squares);
        assert_ne!(taken, Some(Piece::BlackKing), "{}\n{:?}", m, self.squares);

        let mut castle_mask = 0;

        let mut reversible = false;

        if ty == Move::TYPE_NORMAL {
            if let Some(taken) = taken {
                self.take_piece(taken, to);
            } else {
                reversible = piece != Piece::WhitePawn && piece != Piece::BlackPawn;
            }
            self.move_piece(piece, from, to);

            if m.is_double_move() {
                self.state.en_passant = from.file();
            }
        } else if ty == Move::TYPE_CASTLE {
            self.move_piece(piece, from, to);

            match to {
                Square::C1 => {
                    self.move_piece(Piece::WhiteRook, Square::A1, Square::D1);
                    castle_mask = ExtraState::WHITE_KING_CASTLE | ExtraState::WHITE_QUEEN_CASTLE;
                }
                Square::G1 => {
                    self.move_piece(Piece::WhiteRook, Square::H1, Square::F1);
                    castle_mask = ExtraState::WHITE_KING_CASTLE | ExtraState::WHITE_QUEEN_CASTLE;
                }
                Square::C8 => {
                    self.move_piece(Piece::BlackRook, Square::A8, Square::D8);
                    castle_mask = ExtraState::BLACK_KING_CASTLE | ExtraState::BLACK_QUEEN_CASTLE;
                }
                Square::G8 => {
                    self.move_piece(Piece::BlackRook, Square::H8, Square::F8);
                    castle_mask = ExtraState::BLACK_KING_CASTLE | ExtraState::BLACK_QUEEN_CASTLE;
                }
                _ => unreachable!(),
            }
        } else if ty == Move::TYPE_PROMOTION {
            debug_assert_eq!(piece, Piece::player_pawn(self.state.player));
            let promote = match m.promotion_piece() {
                Promotion::Queen => Piece::player_queen(self.state.player),
                Promotion::Knight => Piece::player_knight(self.state.player),
                Promotion::Bishop => Piece::player_bishop(self.state.player),
                Promotion::Rook => Piece::player_rook(self.state.player),
            };

            if let Some(taken) = taken {
                self.take_piece(taken, to)
            }
            self.promote_piece(piece, promote, from, to);
        } else if ty == Move::TYPE_EN_PASSANT {
            let (taken_sq, taken_piece) = match self.state.player {
                Player::White => (to - 8i8, Piece::BlackPawn),
                Player::Black => (to + 8i8, Piece::WhitePawn),
            };

            self.take_piece(taken_piece, taken_sq);
            self.move_piece(piece, from, to);
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

        self.chain.move_end(self.state);

        if reversible {
            self.state.move_clock += 1;
        } else {
            self.state.move_clock = 0;
        }

        let res = UnmakeMove {
            mov: m,
            taken,
            state,
        };
        //self.moves.push(res);
        res
    }

    /// Undo a move
    pub fn unmake_move(&mut self, mov: UnmakeMove) {
        //debug_assert_eq!(self.moves.pop(), Some(mov));
        self.chain.undo_move_start(self.state);

        self.state = mov.state;

        let from = mov.mov.from();
        let to = mov.mov.to();
        let ty = mov.mov.ty();
        let piece = self.squares[to].unwrap();

        if ty == Move::TYPE_NORMAL {
            self.move_piece(piece, to, from);

            if let Some(taken) = mov.taken {
                self.untake_piece(taken, to)
            }
        } else if ty == Move::TYPE_CASTLE {
            let king = piece;
            let rook = Piece::player_rook(self.state.player);
            self.move_piece(king, to, from);

            match to {
                Square::C1 => {
                    self.move_piece(rook, Square::D1, Square::A1);
                }
                Square::G1 => {
                    self.move_piece(rook, Square::F1, Square::H1);
                }
                Square::C8 => {
                    self.move_piece(rook, Square::D8, Square::A8);
                }
                Square::G8 => {
                    self.move_piece(rook, Square::F8, Square::H8);
                }
                _ => unreachable!(),
            }
        } else if ty == Move::TYPE_PROMOTION {
            let piece = Piece::player_pawn(self.state.player);
            let promote = match mov.mov.promotion_piece() {
                Promotion::Queen => Piece::player_queen(self.state.player),
                Promotion::Knight => Piece::player_knight(self.state.player),
                Promotion::Bishop => Piece::player_bishop(self.state.player),
                Promotion::Rook => Piece::player_rook(self.state.player),
            };

            self.unpromote_piece(piece, promote, from, to);

            if let Some(taken) = mov.taken {
                self.untake_piece(taken, to);
            }
        } else if ty == Move::TYPE_EN_PASSANT {
            let m: i8 = match self.state.player {
                Player::Black => 8,
                Player::White => -8,
            };
            let taken_sq = to + m;
            let taken = mov.taken.unwrap();

            self.move_piece(piece, to, from);
            self.untake_piece(taken, taken_sq);
        }

        self.state = mov.state;
    }

    #[inline(always)]
    pub fn on(&self, square: Square) -> Option<Piece> {
        self.squares[square]
    }
}

impl<C: MoveChain + Debug> Debug for Board<C> {
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
            .field("chain", &self.chain)
            .finish()
    }
}

impl<C: MoveChain + Debug> fmt::Display for Board<C> {
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
