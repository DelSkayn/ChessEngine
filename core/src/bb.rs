use super::{Direction, Square};
use std::{
    fmt::{self, Debug},
    iter::{IntoIterator, Iterator},
    ops::{
        self, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, ShlAssign,
        Shr, ShrAssign,
    },
};

/// A single bit board.
#[derive(Eq, PartialEq, Clone, Copy, Default)]
pub struct BB(pub u64);

impl BB {
    pub const FILE_A: BB = BB(0x0101010101010101);
    pub const FILE_B: BB = BB(0x0101010101010101 << 1);
    pub const FILE_C: BB = BB(0x0101010101010101 << 2);
    pub const FILE_D: BB = BB(0x0101010101010101 << 3);
    pub const FILE_E: BB = BB(0x0101010101010101 << 4);
    pub const FILE_F: BB = BB(0x0101010101010101 << 5);
    pub const FILE_G: BB = BB(0x0101010101010101 << 6);
    pub const FILE_H: BB = BB(0x0101010101010101 << 7);

    pub const RANK_1: BB = BB(0xff);
    pub const RANK_2: BB = BB(0xff << 8);
    pub const RANK_3: BB = BB(0xff << 16);
    pub const RANK_4: BB = BB(0xff << 24);
    pub const RANK_5: BB = BB(0xff << 32);
    pub const RANK_6: BB = BB(0xff << 40);
    pub const RANK_7: BB = BB(0xff << 48);
    pub const RANK_8: BB = BB(0xff << 56);

    pub const A1: BB = BB::square(Square::A1);
    pub const B1: BB = BB::square(Square::B1);
    pub const C1: BB = BB::square(Square::C1);
    pub const D1: BB = BB::square(Square::D1);
    pub const E1: BB = BB::square(Square::E1);
    pub const F1: BB = BB::square(Square::F1);
    pub const G1: BB = BB::square(Square::G1);
    pub const H1: BB = BB::square(Square::H1);

    pub const A8: BB = BB::square(Square::A8);
    pub const B8: BB = BB::square(Square::B8);
    pub const C8: BB = BB::square(Square::C8);
    pub const D8: BB = BB::square(Square::D8);
    pub const E8: BB = BB::square(Square::E8);
    pub const F8: BB = BB::square(Square::F8);
    pub const G8: BB = BB::square(Square::G8);
    pub const H8: BB = BB::square(Square::H8);

    pub const WHITE_KING_CASTLE_MASK: BB = BB(0b1100000);
    pub const WHITE_QUEEN_CASTLE_MASK: BB = BB(0b1110);

    pub const EMPTY: BB = BB(0);
    pub const FULL: BB = BB(0xFFFF_FFFF_FFFF_FFFF);

    pub const WHITE_SQUARES: BB = BB(0x55AA55AA_55AA55AA);
    pub const BLACK_SQUARES: BB = BB(0xAA55AA55_AA55AA55);

    /// Returns a board with only the given square enabled
    #[inline]
    pub const fn square(s: Square) -> Self {
        BB(1 << s.get())
    }

    /// Returns a board with no squares enabled
    #[inline]
    pub const fn empty() -> Self {
        BB(0)
    }

    /// Returns a board wth the given square enabled
    #[inline]
    pub fn set(mut self, s: Square) -> Self {
        self.0 |= 1 << s.get();
        self
    }

    /// Returns a board wth the given square disabled
    #[inline]
    pub fn unset(mut self, s: Square) -> Self {
        self.0 &= !(1 << s.get());
        self
    }

    #[inline(always)]
    pub fn const_shift<const S: i8>(self) -> Self {
        if S < 0 {
            self >> (-S) as u8
        } else {
            self << S as u8
        }
    }

    /// Shift the board in a given direction, filters out squares which would wrap.
    #[inline(always)]
    pub fn shift(self, direction: Direction) -> Self {
        match direction {
            Direction::NW => (self & !BB::FILE_A) << 7,
            Direction::N => self << 8,
            Direction::NE => (self & !BB::FILE_H) << 9,
            Direction::E => (self & !BB::FILE_H) << 1,
            Direction::SE => (self & !BB::FILE_H) >> 7,
            Direction::S => self >> 8,
            Direction::SW => (self & !BB::FILE_A) >> 9,
            Direction::W => (self & !BB::FILE_A) >> 1,
        }
    }

    /// Mirror the position of the board.
    pub fn flip(self) -> Self {
        Self(self.0.reverse_bits())
    }

    /// Are any squares enabled
    #[inline]
    pub fn any(self) -> bool {
        self.0 != 0
    }

    /// Are all squares disabled
    #[inline]
    pub fn none(self) -> bool {
        self.0 == 0
    }

    /// Returns the number of enabled squares
    #[inline]
    pub fn count(self) -> u8 {
        self.0.count_ones() as u8
    }

    /// Returns an interator over all enabled squares
    #[inline]
    pub fn iter(self) -> BBIter {
        BBIter(self)
    }

    /// Returns an interator over all enabled squares in the reverse direction
    #[inline]
    pub fn iter_rev(self) -> BBIterRev {
        BBIterRev(self)
    }

    /// Returns the square of the first enabled square
    #[inline]
    pub fn first_piece(self) -> Square {
        let res = self.0.trailing_zeros() as u8;
        Square::new(res)
    }

    /// Returns the square of the last enabled square
    #[inline]
    pub fn last_piece(self) -> Square {
        let res = self.0.leading_zeros() as u8;
        Square::new(63 - res)
    }

    /// Enable all squares if the boolean is true
    #[inline]
    pub fn fill(v: bool) -> Self {
        Self(!(v as u64).wrapping_sub(1))
    }

    /// Enable all squares if there is a single value on the board
    #[inline]
    pub fn saturate(self) -> Self {
        Self::fill(self.0 != 0)
    }

    #[inline]
    pub fn get(self) -> u64 {
        self.0
    }
}

impl ops::Sub for BB {
    type Output = BB;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        BB(self.0.wrapping_sub(rhs.0))
    }
}

impl ops::SubAssign for BB {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0.wrapping_sub(rhs.0);
    }
}

impl Debug for BB {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        writeln!(f, "   +----------------+")?;
        for j in (0..8).rev() {
            write!(f, "{}: |", j + 1)?;
            for i in 0..8 {
                if *self & (1u64 << (j * 8 + i)) != BB::empty() {
                    write!(f, "##")?;
                    //write!(f, "\x1b[97;107m. \x1b[0m")?;
                } else {
                    write!(f, "  ")?;
                    //write!(f, "\x1b[0;100m. \x1b[0m")?;
                }
                if i == 7 {
                    write!(f, "|")?;
                }
            }
            writeln!(f)?;
        }
        writeln!(f, "   +----------------+")?;
        writeln!(f, "    a b c d e f g h  ")?;

        Ok(())
    }
}

impl BitAndAssign for BB {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl BitOrAssign for BB {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl BitXorAssign for BB {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

impl BitAnd for BB {
    type Output = Self;

    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        BB(self.0 & rhs.0)
    }
}

impl BitOr for BB {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        BB(self.0 | rhs.0)
    }
}

impl BitXor for BB {
    type Output = Self;

    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self::Output {
        BB(self.0 ^ rhs.0)
    }
}

impl BitAndAssign<u64> for BB {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: u64) {
        self.0 &= rhs
    }
}

impl BitOrAssign<u64> for BB {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: u64) {
        self.0 |= rhs
    }
}

impl BitXorAssign<u64> for BB {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: u64) {
        self.0 ^= rhs
    }
}

impl BitAnd<u64> for BB {
    type Output = Self;

    #[inline(always)]
    fn bitand(self, rhs: u64) -> Self::Output {
        BB(self.0 & rhs)
    }
}

impl BitOr<u64> for BB {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: u64) -> Self::Output {
        BB(self.0 | rhs)
    }
}

impl BitXor<u64> for BB {
    type Output = Self;

    #[inline(always)]
    fn bitxor(self, rhs: u64) -> Self::Output {
        BB(self.0 ^ rhs)
    }
}

impl Shl<u8> for BB {
    type Output = Self;

    #[inline(always)]
    fn shl(self, rhs: u8) -> Self::Output {
        BB(self.0 << rhs)
    }
}

impl ShlAssign<u8> for BB {
    #[inline(always)]
    fn shl_assign(&mut self, rhs: u8) {
        self.0 <<= rhs
    }
}

impl Shr<u8> for BB {
    type Output = Self;

    #[inline(always)]
    fn shr(self, rhs: u8) -> Self::Output {
        BB(self.0 >> rhs)
    }
}

impl ShrAssign<u8> for BB {
    #[inline(always)]
    fn shr_assign(&mut self, rhs: u8) {
        self.0 >>= rhs
    }
}

impl Not for BB {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self::Output {
        BB(!self.0)
    }
}

impl IntoIterator for BB {
    type IntoIter = BBIter;
    type Item = Square;

    #[inline(always)]
    fn into_iter(self) -> BBIter {
        self.iter()
    }
}

pub struct BBIter(BB);

impl Iterator for BBIter {
    type Item = Square;

    #[inline(always)]
    fn next(&mut self) -> Option<Square> {
        if self.0.none() {
            return None;
        }

        let idx = self.0 .0.trailing_zeros();
        let res = Square::new(idx as u8);
        self.0 ^= BB::square(res);
        Some(res)
    }
}

pub struct BBIterRev(BB);

impl Iterator for BBIterRev {
    type Item = Square;

    fn next(&mut self) -> Option<Square> {
        if self.0.none() {
            return None;
        }

        let idx = 63 - self.0 .0.leading_zeros();
        let res = Square::new(idx as u8);
        self.0 ^= BB::square(res);
        Some(res)
    }
}
