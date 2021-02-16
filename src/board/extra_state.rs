use super::Square;

use std::{
    fmt::{self, Debug},
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign},
};

#[derive(Eq, PartialEq, Clone, Copy)]
pub struct ExtraState(u16);

impl ExtraState {
    pub const BLACK_MOVE: ExtraState = ExtraState(1 << 6);

    pub const WHITE_KING_CASTLE: ExtraState = ExtraState(1 << 7);
    pub const WHITE_QUEEN_CASTLE: ExtraState = ExtraState(1 << 8);

    pub const BLACK_KING_CASTLE: ExtraState = ExtraState(1 << 9);
    pub const BLACK_QUEEN_CASTLE: ExtraState = ExtraState(1 << 10);

    pub const CHECK: ExtraState = ExtraState(1 << 11);
    pub const DOUBLE_CHECK: ExtraState = ExtraState(1 << 12);

    pub const EN_PASSANT_MASK: u16 = 0b111111;

    pub const fn empty() -> Self {
        ExtraState(ExtraState::EN_PASSANT_MASK)
    }

    #[inline]
    pub const fn any(self) -> bool {
        self.0 != 0
    }

    #[inline]
    pub const fn none(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub const fn white_move(self) -> bool {
        (self.0 & ExtraState::BLACK_MOVE.0) == 0
    }

    #[inline]
    pub const fn set_en_passant(mut self, which: Square) -> Self {
        self.0 |= which.0 as u16 & ExtraState::EN_PASSANT_MASK;
        self
    }

    #[inline]
    pub const fn get_en_passant(self) -> Option<Square> {
        let res = self.0 & ExtraState::EN_PASSANT_MASK;
        if res == ExtraState::EN_PASSANT_MASK {
            None
        } else {
            Some(Square(res as u8))
        }
    }
}

impl BitAndAssign for ExtraState {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl BitOrAssign for ExtraState {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl BitXorAssign for ExtraState {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

impl BitAnd for ExtraState {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        ExtraState(self.0 & rhs.0)
    }
}

impl BitOr for ExtraState {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        ExtraState(self.0 | rhs.0)
    }
}

impl BitXor for ExtraState {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        ExtraState(self.0 ^ rhs.0)
    }
}

impl Debug for ExtraState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExtraState")
            .field("black_move", &(*self & ExtraState::BLACK_MOVE).any())
            .field(
                "white_king_castle",
                &(*self & ExtraState::WHITE_KING_CASTLE).any(),
            )
            .field(
                "white_queen_castle",
                &(*self & ExtraState::WHITE_QUEEN_CASTLE).any(),
            )
            .field(
                "black_king_castle",
                &(*self & ExtraState::BLACK_KING_CASTLE).any(),
            )
            .field(
                "black_queen_castle",
                &(*self & ExtraState::BLACK_QUEEN_CASTLE).any(),
            )
            .field("en_passant", &self.get_en_passant())
            .finish()
    }
}
