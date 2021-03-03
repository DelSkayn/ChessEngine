use super::Square;

use std::{
    fmt::{self, Debug},
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign},
};

#[derive(Eq, PartialEq, Clone, Copy)]
pub struct ExtraState(u16);

impl ExtraState {
    pub const BLACK_MOVE: ExtraState = ExtraState(1 << 7);

    pub const WHITE_KING_CASTLE: ExtraState = ExtraState(1 << 8);
    pub const WHITE_QUEEN_CASTLE: ExtraState = ExtraState(1 << 9);

    pub const BLACK_KING_CASTLE: ExtraState = ExtraState(1 << 10);
    pub const BLACK_QUEEN_CASTLE: ExtraState = ExtraState(1 << 11);

    pub const EN_PASSANT_MASK: u16 = 0b011111;
    pub const EN_PASSANT_PRESENT: u16 = 0b100000;

    pub const fn empty() -> Self {
        ExtraState(ExtraState::EN_PASSANT_PRESENT)
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

    pub fn make_move(self) -> Self {
        Self(self.0 ^ ExtraState::BLACK_MOVE.0)
    }

    #[inline]
    pub const fn set_en_passant(self, which: Square) -> Self {
        let res = self.0;
        let res = (res & !ExtraState::EN_PASSANT_MASK)
            | (which.0 as u16 & (ExtraState::EN_PASSANT_MASK | ExtraState::EN_PASSANT_PRESENT));
        Self(res)
    }

    #[inline]
    pub const fn get_en_passant(self) -> Option<Square> {
        if self.0 & ExtraState::EN_PASSANT_PRESENT == 0 {
            Some(Square((self.0 & ExtraState::EN_PASSANT_MASK) as u8))
        } else {
            None
        }
    }

    pub fn flip(self) -> Self {
        let castle_w = ((0b11 << 8) & self.0) << 2;
        let castle_b = ((0b11 << 10) & self.0) >> 2;
        let en_passant = 63 - (self.0 & Self::EN_PASSANT_MASK);
        let en_passant_present = self.0 & Self::EN_PASSANT_PRESENT;
        let black_move = (self.0 & Self::BLACK_MOVE.0) ^ Self::BLACK_MOVE.0;
        let res = Self(castle_w | castle_b | en_passant_present | en_passant | black_move);
        res
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
