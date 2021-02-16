use super::Square;

use std::{
    fmt::{self, Debug},
    iter::Iterator,
    ops::{
        BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, ShlAssign, Shr,
        ShrAssign,
    },
};

#[derive(Eq, PartialEq, Clone, Copy)]
pub struct BB(pub u64);

impl BB {
    pub const fn square(s: Square) -> Self {
        BB(1 << s.0)
    }

    #[inline]
    pub const fn empty() -> Self {
        BB(0)
    }

    #[inline]
    pub fn get(self, value: u8) -> bool {
        self.0 & (1 << value) != 0
    }

    #[inline]
    pub fn set(mut self, value: u8) -> Self {
        self.0 |= 1 << value;
        self
    }

    #[inline]
    pub fn unset(mut self, value: u8) -> Self {
        self.0 &= !(1 << value);
        self
    }

    #[inline]
    pub fn any(self) -> bool {
        self.0 != 0
    }

    #[inline]
    pub fn none(self) -> bool {
        self.0 == 0
    }
}

impl Debug for BB {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        writeln!(f, "   +-----------------+")?;
        for j in 0..8 {
            write!(f, "{}: | ", 8 - j)?;
            for i in 0..8 {
                if *self & (1u64 << j * 8 + i) != BB::empty() {
                    write!(f, "1 ")?;
                } else {
                    write!(f, "0 ")?;
                }
                if i == 7 {
                    write!(f, "|")?;
                }
            }
            writeln!(f)?;
        }
        writeln!(f, "   +-----------------+")?;
        writeln!(f, "     a b c d e f g h  ")?;

        Ok(())
    }
}

impl BitAndAssign for BB {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl BitOrAssign for BB {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl BitXorAssign for BB {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

impl BitAnd for BB {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        BB(self.0 & rhs.0)
    }
}

impl BitOr for BB {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        BB(self.0 | rhs.0)
    }
}

impl BitXor for BB {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        BB(self.0 ^ rhs.0)
    }
}

impl BitAndAssign<u64> for BB {
    fn bitand_assign(&mut self, rhs: u64) {
        self.0 &= rhs
    }
}

impl BitOrAssign<u64> for BB {
    fn bitor_assign(&mut self, rhs: u64) {
        self.0 |= rhs
    }
}

impl BitXorAssign<u64> for BB {
    fn bitxor_assign(&mut self, rhs: u64) {
        self.0 ^= rhs
    }
}

impl BitAnd<u64> for BB {
    type Output = Self;

    fn bitand(self, rhs: u64) -> Self::Output {
        BB(self.0 & rhs)
    }
}

impl BitOr<u64> for BB {
    type Output = Self;

    fn bitor(self, rhs: u64) -> Self::Output {
        BB(self.0 | rhs)
    }
}

impl BitXor<u64> for BB {
    type Output = Self;

    fn bitxor(self, rhs: u64) -> Self::Output {
        BB(self.0 ^ rhs)
    }
}

impl Shl for BB {
    type Output = Self;

    fn shl(self, rhs: Self) -> Self::Output {
        BB(self.0 << rhs.0)
    }
}

impl Shl<u64> for BB {
    type Output = Self;

    fn shl(self, rhs: u64) -> Self::Output {
        BB(self.0 << rhs)
    }
}

impl ShlAssign for BB {
    fn shl_assign(&mut self, rhs: Self) {
        self.0 <<= rhs.0
    }
}

impl ShlAssign<u64> for BB {
    fn shl_assign(&mut self, rhs: u64) {
        self.0 <<= rhs
    }
}

impl Shr for BB {
    type Output = Self;

    fn shr(self, rhs: Self) -> Self::Output {
        BB(self.0 >> rhs.0)
    }
}

impl Shr<u64> for BB {
    type Output = Self;

    fn shr(self, rhs: u64) -> Self::Output {
        BB(self.0 >> rhs)
    }
}

impl ShrAssign for BB {
    fn shr_assign(&mut self, rhs: Self) {
        self.0 <<= rhs.0
    }
}

impl ShrAssign<u64> for BB {
    fn shr_assign(&mut self, rhs: u64) {
        self.0 <<= rhs
    }
}

impl Not for BB {
    type Output = Self;

    fn not(self) -> Self::Output {
        BB(!self.0)
    }
}

pub struct BBIter(BB);

impl Iterator for BBIter {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.none() {
            return None;
        }

        let idx = self.0 .0.leading_zeros();
        let res = Square(idx as u8);
        self.0 ^= BB::square(res);
        Some(res)
    }
}
