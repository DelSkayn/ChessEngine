use super::Square;
use std::fmt;

/// A move on the board.
///
/// Encoded as from to with possible extra info regarding promotions, en passants or castles.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Move(u16);

impl Move {
    pub const INVALID: Move = Move(0xffff);

    pub const FROM_MASK: u16 = 0b111_111;
    pub const TO_MASK: u16 = 0b111_111 << 6;

    pub const TYPE_MASK: u16 = 0b11 << 12;
    pub const TYPE_CASTLE: u16 = 1 << 12;
    pub const TYPE_PROMOTION: u16 = 2 << 12;
    pub const TYPE_EN_PASSANT: u16 = 3 << 12;
    pub const TYPE_NORMAL: u16 = 0;

    pub const PROMOTION_MASK: u16 = 0b11 << 14;

    pub const PROMOTION_QUEEN: u16 = 0 << 14;
    pub const PROMOTION_KNIGHT: u16 = 1 << 14;
    pub const PROMOTION_ROOK: u16 = 2 << 14;
    pub const PROMOTION_BISHOP: u16 = 3 << 14;

    pub const DOUBLE_MOVE_PAWN: u16 = 1 << 14;

    pub fn from_u16(v: u16) -> Self {
        Move(v)
    }

    pub fn to_u16(self) -> u16 {
        self.0
    }

    #[inline]
    pub fn new(from: Square, to: Square, ty: u16, promotion: u16) -> Self {
        Self(from.get() as u16 | (to.get() as u16) << 6 | ty | promotion)
    }

    #[inline]
    pub fn normal(from: Square, to: Square) -> Self {
        Self(from.get() as u16 | (to.get() as u16) << 6)
    }

    #[inline]
    pub fn double_pawn(from: Square, to: Square) -> Self {
        Self(Self::DOUBLE_MOVE_PAWN | from.get() as u16 | (to.get() as u16) << 6)
    }

    #[inline]
    pub fn castle(from: Square, to: Square) -> Self {
        Self(from.get() as u16 | (to.get() as u16) << 6 | Self::TYPE_CASTLE)
    }

    #[inline]
    pub fn promotion(from: Square, to: Square, promotion: u16) -> Self {
        debug_assert!(promotion >= Self::PROMOTION_QUEEN && promotion <= Self::PROMOTION_BISHOP);
        Self(from.get() as u16 | (to.get() as u16) << 6 | Self::TYPE_PROMOTION | promotion)
    }

    pub fn en_passant(from: Square, to: Square) -> Self {
        Self(from.get() as u16 | (to.get() as u16) << 6 | Self::TYPE_EN_PASSANT)
    }

    #[inline]
    pub fn from(self) -> Square {
        Square::new((self.0 & Self::FROM_MASK) as u8)
    }

    #[inline]
    pub fn to(self) -> Square {
        Square::new(((self.0 & Self::TO_MASK) >> 6) as u8)
    }

    #[inline]
    pub fn ty(self) -> u16 {
        self.0 & Self::TYPE_MASK
    }

    #[inline]
    pub fn is_double_move(self) -> bool {
        debug_assert!(self.ty() == Self::TYPE_NORMAL);
        self.0 & Self::PROMOTION_MASK != 0
    }

    #[inline]
    pub fn promotion_piece(self) -> u16 {
        debug_assert_eq!(self.ty(), Self::TYPE_PROMOTION);
        self.0 & Self::PROMOTION_MASK
    }

    pub fn from_name(name: &str) -> Option<Self> {
        let from = Square::from_name(&name[0..2])?;
        let to = Square::from_name(&name[2..])?;
        Some(Self::normal(from, to))
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Move")
            .field("value", &self.0)
            .field("notation", &format!("{}", self))
            .field("ty", &(self.ty() >> 12))
            .field("to", &self.to())
            .field("from", &self.from())
            .field(
                "promotion",
                &if self.ty() == Self::TYPE_PROMOTION {
                    Some(self.promotion_piece() >> 14)
                } else {
                    None
                },
            )
            .finish()
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == Self::INVALID {
            return write!(f, "INVALID");
        }

        if self.ty() == Self::TYPE_CASTLE {
            if self.to() == Square::C1 || self.to() == Square::C8 {
                return write!(f, "O-O({},{})", self.from(), self.to());
            } else {
                return write!(f, "O-O-O({},{})", self.from(), self.to());
            }
        }
        write!(f, "{}{}", self.from(), self.to())?;
        if self.ty() == Self::TYPE_PROMOTION {
            let piece = match self.promotion_piece() {
                Self::PROMOTION_QUEEN => "Q",
                Self::PROMOTION_KNIGHT => "K",
                Self::PROMOTION_ROOK => "R",
                Self::PROMOTION_BISHOP => "B",
                _ => unreachable!(),
            };
            return write!(f, "={}", piece);
        }
        if self.ty() == Self::TYPE_EN_PASSANT {
            return write!(f, "e.p.");
        }

        Ok(())
    }
}
