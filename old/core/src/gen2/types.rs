//! Kinds of move generation.

use crate::{bb::BB, Direction, Piece, Square};

pub trait GenType {
    const QUIET: bool;
    const CHECKS: bool;
    const LEGAL: bool;
}

pub mod gen_type {
    use super::GenType;
    pub struct Captures;
    pub struct All;
    pub struct AllPseudo;
    pub struct CapturesChecksPseudo;

    impl GenType for Captures {
        const QUIET: bool = false;
        const LEGAL: bool = true;
        const CHECKS: bool = false;
    }

    impl GenType for All {
        const QUIET: bool = true;
        const LEGAL: bool = true;
        const CHECKS: bool = false;
    }

    impl GenType for AllPseudo {
        const QUIET: bool = true;
        const LEGAL: bool = false;
        const CHECKS: bool = false;
    }

    impl GenType for CapturesChecksPseudo {
        const QUIET: bool = false;
        const LEGAL: bool = false;
        const CHECKS: bool = true;
    }
}

pub trait Player {
    type Opponent: Player;

    const ATTACK_LEFT: Direction;
    const ATTACK_RIGHT: Direction;
    const LEFT: Direction;
    const RIGHT: Direction;
    const PAWN_MOVE: Direction;

    const RANK_7: BB;
    const RANK_8: BB;
    const RANK_3: BB;
    const RANK_5: BB;

    const KING: Piece;
    const QUEEN: Piece;
    const ROOK: Piece;
    const KNIGHT: Piece;
    const BISHOP: Piece;
    const PAWN: Piece;

    const FLAG_SHIFT: u8;
    const CASTLE_SHIFT: u8;

    const CASTLE_FROM: Square;
    const CASTLE_KING_TO: Square;
    const CASTLE_QUEEN_TO: Square;

    const IS_BLACK: bool;
}

pub struct White;
pub struct Black;

impl Player for White {
    type Opponent = Black;

    const ATTACK_LEFT: Direction = Direction::NW;
    const ATTACK_RIGHT: Direction = Direction::NE;
    const LEFT: Direction = Direction::W;
    const RIGHT: Direction = Direction::E;
    const PAWN_MOVE: Direction = Direction::N;

    const RANK_7: BB = BB::RANK_7;
    const RANK_8: BB = BB::RANK_8;
    const RANK_3: BB = BB::RANK_3;
    const RANK_5: BB = BB::RANK_5;

    const KING: Piece = Piece::WhiteKing;
    const QUEEN: Piece = Piece::WhiteQueen;
    const ROOK: Piece = Piece::WhiteRook;
    const KNIGHT: Piece = Piece::WhiteKnight;
    const BISHOP: Piece = Piece::WhiteBishop;
    const PAWN: Piece = Piece::WhitePawn;

    const FLAG_SHIFT: u8 = 0;
    const CASTLE_SHIFT: u8 = 0;

    const CASTLE_FROM: Square = Square::E1;
    const CASTLE_KING_TO: Square = Square::G1;
    const CASTLE_QUEEN_TO: Square = Square::C1;

    const IS_BLACK: bool = false;
}

impl Player for Black {
    type Opponent = White;

    const ATTACK_LEFT: Direction = Direction::SE;
    const ATTACK_RIGHT: Direction = Direction::SW;
    const LEFT: Direction = Direction::E;
    const RIGHT: Direction = Direction::W;
    const PAWN_MOVE: Direction = Direction::S;

    const RANK_7: BB = BB::RANK_2;
    const RANK_8: BB = BB::RANK_1;
    const RANK_3: BB = BB::RANK_6;
    const RANK_5: BB = BB::RANK_4;

    const KING: Piece = Piece::BlackKing;
    const QUEEN: Piece = Piece::BlackQueen;
    const ROOK: Piece = Piece::BlackRook;
    const KNIGHT: Piece = Piece::BlackKnight;
    const BISHOP: Piece = Piece::BlackBishop;
    const PAWN: Piece = Piece::BlackPawn;

    const FLAG_SHIFT: u8 = 2;
    const CASTLE_SHIFT: u8 = 8 * 7;

    const CASTLE_FROM: Square = Square::E8;
    const CASTLE_KING_TO: Square = Square::G8;
    const CASTLE_QUEEN_TO: Square = Square::C8;
    const IS_BLACK: bool = true;
}
