use crate::{Direction, Piece, Player, Square, BB};

pub trait PlayerType {
    type Opponent: PlayerType;

    const ATTACK_LEFT: Direction;
    const ATTACK_RIGHT: Direction;
    const PAWN_MOVE: Direction;

    const RANK_7: BB;
    const RANK_8: BB;
    const RANK_3: BB;

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

    const VALUE: Player;
}

pub struct White;
pub struct Black;

impl PlayerType for White {
    type Opponent = Black;

    const ATTACK_LEFT: Direction = Direction::NW;
    const ATTACK_RIGHT: Direction = Direction::NE;
    const PAWN_MOVE: Direction = Direction::N;

    const RANK_7: BB = BB::RANK_7;
    const RANK_8: BB = BB::RANK_8;
    const RANK_3: BB = BB::RANK_3;

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

    const VALUE: Player = Player::White;
}

impl PlayerType for Black {
    type Opponent = White;

    const ATTACK_LEFT: Direction = Direction::SE;
    const ATTACK_RIGHT: Direction = Direction::SW;
    const PAWN_MOVE: Direction = Direction::S;

    const RANK_7: BB = BB::RANK_2;
    const RANK_8: BB = BB::RANK_1;
    const RANK_3: BB = BB::RANK_6;

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

    const VALUE: Player = Player::Black;
}
