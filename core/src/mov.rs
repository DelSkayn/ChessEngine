use super::Square;
use std::{fmt, mem};

#[repr(u16)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Promotion {
    Queen = 0,
    Knight = 1 << 14,
    Rook = 2 << 14,
    Bishop = 3 << 14,
}

#[repr(u16)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MoveKind {
    Normal = 0,
    Castle = 1 << 12,
    Promotion = 2 << 12,
    EnPassant = 3 << 12,
}

impl fmt::Display for Promotion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Promotion::Queen => write!(f, "q"),
            Promotion::Knight => write!(f, "k"),
            Promotion::Rook => write!(f, "r"),
            Promotion::Bishop => write!(f, "b"),
        }
    }
}

/// A move on the board.
///
/// Encoded as from to with possible extra info regarding promotions, en passants or castles.
#[derive(Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    pub fn new(from: Square, to: Square, ty: u16, promotion: Promotion) -> Self {
        Self(from.get() as u16 | (to.get() as u16) << 6 | ty | promotion as u16)
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
    pub fn promotion(from: Square, to: Square, promotion: Promotion) -> Self {
        Self(from.get() as u16 | (to.get() as u16) << 6 | Self::TYPE_PROMOTION | promotion as u16)
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
    pub fn kind(self) -> MoveKind {
        unsafe { std::mem::transmute(self.0 & Self::TYPE_MASK) }
    }

    #[inline]
    pub fn is_double_move(self) -> bool {
        debug_assert!(self.ty() == Self::TYPE_NORMAL);
        self.0 & Self::PROMOTION_MASK != 0
    }

    #[inline]
    pub fn promotion_piece(self) -> Promotion {
        debug_assert_eq!(self.ty(), Self::TYPE_PROMOTION);
        let raw = self.0 & Self::PROMOTION_MASK;
        // Raw should only be a possible value from Promotion enum
        unsafe { mem::transmute::<_, Promotion>(raw) }
    }

    #[inline]
    pub fn get_promotion(self) -> Option<Promotion> {
        if let MoveKind::Promotion = self.kind() {
            Some(self.promotion_piece())
        } else {
            None
        }
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
                    Some(self.promotion_piece())
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
                Promotion::Queen => "Q",
                Promotion::Knight => "K",
                Promotion::Rook => "R",
                Promotion::Bishop => "B",
            };
            return write!(f, "={}", piece);
        }
        if self.ty() == Self::TYPE_EN_PASSANT {
            return write!(f, "e.p.");
        }

        Ok(())
    }
}
