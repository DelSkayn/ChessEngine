use std::fmt::Write;

use crate::Player;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(u8)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SquareContent {
    WhiteKing = 0b0000,
    BlackKing = 0b0001,

    WhitePawn = 0b0010,
    BlackPawn = 0b0011,

    WhiteBishop = 0b0100,
    BlackBishop = 0b0101,

    WhiteKnight = 0b0110,
    BlackKnight = 0b0111,

    WhiteRook = 0b1000,
    BlackRook = 0b1001,

    WhiteQueen = 0b1010,
    BlackQueen = 0b1011,

    Empty = 0b1100,
}

impl std::fmt::Display for SquareContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(self.to_char())
    }
}

impl SquareContent {
    /// Create from u8 representation
    pub fn from_u8(v: u8) -> Self {
        assert!(v <= SquareContent::Empty as u8);
        unsafe { Self::from_u8_unchecked(v) }
    }

    /// Create from u8 representation without checking if value is valid.
    ///
    /// # Safety
    /// Caller must ensure that the give representation is valid.
    pub unsafe fn from_u8_unchecked(v: u8) -> Self {
        debug_assert!(v <= SquareContent::Empty as u8);
        std::mem::transmute(v)
    }

    pub fn to_piece(self) -> Option<Piece> {
        if let SquareContent::Empty = self {
            return None;
        }
        Some(unsafe { Piece::from_u8_unchecked(self as u8) })
    }

    pub const fn to_char(self) -> char {
        match self {
            Self::WhiteKing => 'K',
            Self::BlackKing => 'k',
            Self::WhiteQueen => 'Q',
            Self::BlackQueen => 'q',
            Self::WhiteRook => 'R',
            Self::BlackRook => 'r',
            Self::WhiteBishop => 'B',
            Self::BlackBishop => 'b',
            Self::WhiteKnight => 'N',
            Self::BlackKnight => 'n',
            Self::WhitePawn => 'P',
            Self::BlackPawn => 'p',
            Self::Empty => ' ',
        }
    }
}

impl From<Piece> for SquareContent {
    fn from(value: Piece) -> Self {
        unsafe { Self::from_u8_unchecked(value as u8) }
    }
}

/// All the possible pieces on the board
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(u8)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Piece {
    WhiteKing = 0b0000,
    BlackKing = 0b0001,

    WhitePawn = 0b0010,
    BlackPawn = 0b0011,

    WhiteBishop = 0b0100,
    BlackBishop = 0b0101,

    WhiteKnight = 0b0110,
    BlackKnight = 0b0111,

    WhiteRook = 0b1000,
    BlackRook = 0b1001,

    WhiteQueen = 0b1010,
    BlackQueen = 0b1011,
}

impl Piece {
    /// The first piece in the representation
    pub const START: Self = Self::WhiteKing;
    /// The last piece in the representation
    pub const END: Self = Self::BlackQueen;
    /// The amount of pieces
    pub const AMOUNT: u8 = Self::END as u8 + 1;

    #[inline]
    pub const fn all() -> AllIter {
        AllIter { cur: 0 }
    }

    #[inline]
    pub const fn player_pieces(player: Player) -> PlayerIter {
        PlayerIter { cur: player as u8 }
    }

    /// Always flip the color of the piece
    #[inline]
    pub const fn flip(self) -> Self {
        unsafe { Self::from_u8_unchecked(self as u8 ^ 0b1) }
    }

    /// Flip the color of the piece if the condition is true
    #[inline]
    pub const fn cond_flip(self, cond: bool) -> Self {
        unsafe { Self::from_u8_unchecked(self as u8 ^ cond as u8) }
    }

    /// Return an iterator with all the peices a pawn can promote to of either the black player  or the white player
    #[inline]
    pub fn player_promote_pieces(player: Player) -> PlayerIter {
        PlayerIter {
            cur: Self::WhiteBishop as u8 | player as u8,
        }
    }

    /// Returns the king for the given player
    #[inline]
    pub const fn player_king(player: Player) -> Self {
        unsafe { Piece::from_u8_unchecked(Piece::WhiteKing as u8 | player as u8) }
    }

    /// Returns the queen for the given player
    #[inline]
    pub const fn player_queen(player: Player) -> Self {
        unsafe { Piece::from_u8_unchecked(Piece::WhiteQueen as u8 | player as u8) }
    }

    /// Returns the rook for the given player
    #[inline]
    pub const fn player_rook(player: Player) -> Self {
        unsafe { Piece::from_u8_unchecked(Piece::WhiteRook as u8 | player as u8) }
    }

    /// Returns the bishop for the given player
    #[inline]
    pub const fn player_bishop(player: Player) -> Self {
        unsafe { Piece::from_u8_unchecked(Piece::WhiteBishop as u8 | player as u8) }
    }

    /// Returns the knight for the given player
    #[inline]
    pub const fn player_knight(player: Player) -> Self {
        unsafe { Piece::from_u8_unchecked(Piece::WhiteKnight as u8 | player as u8) }
    }

    /// Returns the pawn for the given player
    #[inline]
    pub const fn player_pawn(player: Player) -> Self {
        unsafe { Piece::from_u8_unchecked(Piece::WhitePawn as u8 | player as u8) }
    }

    pub const fn to_char(self) -> char {
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
            Piece::WhitePawn => 'P',
            Piece::BlackPawn => 'p',
        }
    }

    pub const fn from_char(c: char) -> Option<Self> {
        match c {
            'K' => Some(Piece::WhiteKing),
            'k' => Some(Piece::BlackKing),
            'Q' => Some(Piece::WhiteQueen),
            'q' => Some(Piece::BlackQueen),
            'R' => Some(Piece::WhiteRook),
            'r' => Some(Piece::BlackRook),
            'B' => Some(Piece::WhiteBishop),
            'b' => Some(Piece::BlackBishop),
            'N' => Some(Piece::WhiteKnight),
            'n' => Some(Piece::BlackKnight),
            'P' => Some(Piece::WhitePawn),
            'p' => Some(Piece::BlackPawn),
            _ => None,
        }
    }

    #[inline]
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

    /// Create a piece from its u8 representation
    ///
    /// # Safety
    /// Caller must ensure the given value is within 0..=11
    pub const unsafe fn from_u8_unchecked(v: u8) -> Self {
        debug_assert!(v <= Self::END as u8);
        std::mem::transmute(v)
    }

    /// Returns to which player this piece belongs.
    #[inline]
    pub const fn player(self) -> Player {
        if self.white() {
            Player::White
        } else {
            Player::Black
        }
    }

    /// Returns to if the piece is a white piece.
    #[inline]
    pub const fn white(self) -> bool {
        self as u8 & 0b1 == 0
    }
}

#[derive(Debug)]
pub struct AllIter {
    cur: u8,
}

impl Iterator for AllIter {
    type Item = Piece;

    #[inline]
    fn next(&mut self) -> Option<Piece> {
        if self.cur == Piece::AMOUNT {
            return None;
        }
        let res = unsafe { Piece::from_u8_unchecked(self.cur) };
        self.cur += 1;
        Some(res)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let res = (Piece::AMOUNT - self.cur) as usize;
        (res, Some(res))
    }
}

impl ExactSizeIterator for AllIter {
    #[inline]
    fn len(&self) -> usize {
        (Piece::AMOUNT - self.cur) as usize
    }
}

#[derive(Debug)]
pub struct PlayerIter {
    cur: u8,
}

impl Iterator for PlayerIter {
    type Item = Piece;

    #[inline]
    fn next(&mut self) -> Option<Piece> {
        if self.cur >= Piece::AMOUNT {
            return None;
        }
        let res = unsafe { Piece::from_u8_unchecked(self.cur) };
        self.cur += 2;
        Some(res)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let res = self.len();
        (res, Some(res))
    }
}

impl ExactSizeIterator for PlayerIter {
    #[inline]
    fn len(&self) -> usize {
        (Piece::AMOUNT + 1 - self.cur) as usize / 2
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn select() {
        assert_eq!(Piece::player_king(Player::White), Piece::WhiteKing);
        assert_eq!(Piece::player_king(Player::Black), Piece::BlackKing);
        assert_eq!(Piece::player_queen(Player::White), Piece::WhiteQueen);
        assert_eq!(Piece::player_queen(Player::Black), Piece::BlackQueen);
        assert_eq!(Piece::player_knight(Player::White), Piece::WhiteKnight);
        assert_eq!(Piece::player_knight(Player::Black), Piece::BlackKnight);
        assert_eq!(Piece::player_bishop(Player::White), Piece::WhiteBishop);
        assert_eq!(Piece::player_bishop(Player::Black), Piece::BlackBishop);
        assert_eq!(Piece::player_rook(Player::White), Piece::WhiteRook);
        assert_eq!(Piece::player_rook(Player::Black), Piece::BlackRook);
        assert_eq!(Piece::player_pawn(Player::White), Piece::WhitePawn);
        assert_eq!(Piece::player_pawn(Player::Black), Piece::BlackPawn);
    }

    #[test]
    fn iter_all() {
        let all: Vec<Piece> = Piece::all().collect();
        assert_eq!(all.len(), 12);
        assert!(all.contains(&Piece::WhiteKing));
        assert!(all.contains(&Piece::BlackKing));

        assert!(all.contains(&Piece::WhitePawn));
        assert!(all.contains(&Piece::BlackPawn));

        assert!(all.contains(&Piece::WhiteBishop));
        assert!(all.contains(&Piece::BlackBishop));

        assert!(all.contains(&Piece::WhiteKnight));
        assert!(all.contains(&Piece::BlackKnight));

        assert!(all.contains(&Piece::WhiteRook));
        assert!(all.contains(&Piece::BlackRook));

        assert!(all.contains(&Piece::WhiteQueen));
        assert!(all.contains(&Piece::BlackQueen));
    }

    #[test]
    fn iter_player() {
        let white: Vec<Piece> = Piece::player_pieces(Player::White).collect();
        assert_eq!(white.len(), 6);
        assert!(white.contains(&Piece::WhiteKing));
        assert!(white.contains(&Piece::WhitePawn));
        assert!(white.contains(&Piece::WhiteBishop));
        assert!(white.contains(&Piece::WhiteKnight));
        assert!(white.contains(&Piece::WhiteRook));
        assert!(white.contains(&Piece::WhiteQueen));

        let black: Vec<Piece> = Piece::player_pieces(Player::Black).collect();
        assert_eq!(black.len(), 6);
        assert!(black.contains(&Piece::BlackKing));
        assert!(black.contains(&Piece::BlackPawn));
        assert!(black.contains(&Piece::BlackBishop));
        assert!(black.contains(&Piece::BlackKnight));
        assert!(black.contains(&Piece::BlackRook));
        assert!(black.contains(&Piece::BlackQueen));
    }

    #[test]
    fn iter_promote() {
        let white: Vec<Piece> = Piece::player_promote_pieces(Player::White).collect();
        println!("{:?}", white);
        assert_eq!(white.len(), 4);
        assert!(white.contains(&Piece::WhiteBishop));
        assert!(white.contains(&Piece::WhiteKnight));
        assert!(white.contains(&Piece::WhiteRook));
        assert!(white.contains(&Piece::WhiteQueen));

        let black: Vec<Piece> = Piece::player_promote_pieces(Player::Black).collect();
        assert_eq!(black.len(), 4);
        assert!(black.contains(&Piece::BlackBishop));
        assert!(black.contains(&Piece::BlackKnight));
        assert!(black.contains(&Piece::BlackRook));
        assert!(black.contains(&Piece::BlackQueen));
    }
}
