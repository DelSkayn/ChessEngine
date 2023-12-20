use crate::Player;
use std::mem;

/// All the possible pieces on the board
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(u8)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    /// Create an iterator iterating between self to and including the given piece
    /// ```
    /// # use chess_core::Piece;
    /// println!("all pieces:");
    /// for p in Piece::WhiteKing.to(Piece::BlackPawn){
    ///     println!("{:?}",p);
    /// }
    pub fn to(self, end: Piece) -> PieceIter {
        PieceIter {
            cur: self as u8,
            end: end as u8,
        }
    }

    /// Flip the color of the piece
    pub fn flip(self, v: bool) -> Self {
        let val = (self as u8 + (v as u8 * 6)) % 12;
        unsafe { mem::transmute(val) }
    }

    /// Return an iterator with all the peices of either the black player  or the white player
    #[inline(always)]
    pub fn player_pieces(black: bool) -> PieceIter {
        let add = if black { 6 } else { 0 };
        PieceIter {
            cur: Piece::WhiteKing as u8 + add,
            end: Piece::WhitePawn as u8 + add,
        }
    }

    /// Return an iterator with all the peices a pawn can promote to of either the black player  or the white player
    #[inline(always)]
    pub fn player_promote_pieces(black: bool) -> PieceIter {
        let add = if black { 6 } else { 0 };
        PieceIter {
            cur: Piece::WhiteQueen as u8 + add,
            end: Piece::WhiteRook as u8 + add,
        }
    }

    #[inline(always)]
    pub fn player_king(player: Player) -> Self {
        match player {
            Player::White => Piece::WhiteKing,
            Player::Black => Piece::BlackKing,
        }
    }

    #[inline(always)]
    pub fn player_queen(player: Player) -> Self {
        match player {
            Player::White => Piece::WhiteQueen,
            Player::Black => Piece::BlackQueen,
        }
    }

    #[inline(always)]
    pub fn player_rook(player: Player) -> Self {
        match player {
            Player::White => Piece::WhiteRook,
            Player::Black => Piece::BlackRook,
        }
    }

    #[inline(always)]
    pub fn player_bishop(player: Player) -> Self {
        match player {
            Player::White => Piece::WhiteBishop,
            Player::Black => Piece::BlackBishop,
        }
    }

    #[inline(always)]
    pub fn player_knight(player: Player) -> Self {
        match player {
            Player::White => Piece::WhiteKnight,
            Player::Black => Piece::BlackKnight,
        }
    }

    #[inline(always)]
    pub fn player_pawn(player: Player) -> Self {
        match player {
            Player::White => Piece::WhitePawn,
            Player::Black => Piece::BlackPawn,
        }
    }

    pub fn to_char(self) -> char {
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

    pub fn player(self) -> Player {
        if self.white() {
            Player::White
        } else {
            Player::Black
        }
    }

    #[inline]
    pub fn white(self) -> bool {
        (self as u8) < 6
    }
}
