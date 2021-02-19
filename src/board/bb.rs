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
    pub const FILE_H: BB = BB(0x0101010101010101);
    pub const FILE_G: BB = BB(0x0101010101010101 >> 1);
    pub const FILE_F: BB = BB(0x0101010101010101 >> 2);
    pub const FILE_E: BB = BB(0x0101010101010101 >> 3);
    pub const FILE_D: BB = BB(0x0101010101010101 >> 4);
    pub const FILE_C: BB = BB(0x0101010101010101 >> 5);
    pub const FILE_B: BB = BB(0x0101010101010101 >> 6);
    pub const FILE_A: BB = BB(0x0101010101010101 >> 7);

    pub const RANK_8: BB = BB(0xff);
    pub const RANK_7: BB = BB(0xff << 8);
    pub const RANK_6: BB = BB(0xff << 16);
    pub const RANK_5: BB = BB(0xff << 24);
    pub const RANK_4: BB = BB(0xff << 32);
    pub const RANK_3: BB = BB(0xff << 40);
    pub const RANK_2: BB = BB(0xff << 48);
    pub const RANK_1: BB = BB(0xff << 52);

    pub const fn square(s: u8) -> Self {
        BB(1 << s)
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

    #[inline(always)]
    pub fn shift(self, value: i8) -> Self {
        if value < 0 {
            self >> (-value) as u8
        } else {
            self << value as u8
        }
    }

    pub fn flip(self) -> Self {
        Self(self.0.reverse_bits())
    }

    #[inline]
    pub fn any(self) -> bool {
        self.0 != 0
    }

    #[inline]
    pub fn none(self) -> bool {
        self.0 == 0
    }

    pub fn iter(self) -> BBIter {
        BBIter(self)
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

impl Shl<u8> for BB {
    type Output = Self;

    fn shl(self, rhs: u8) -> Self::Output {
        BB(self.0 << rhs)
    }
}

impl ShlAssign<u8> for BB {
    fn shl_assign(&mut self, rhs: u8) {
        self.0 <<= rhs
    }
}

impl Shr<u8> for BB {
    type Output = Self;

    fn shr(self, rhs: u8) -> Self::Output {
        BB(self.0 >> rhs)
    }
}

impl ShrAssign<u8> for BB {
    fn shr_assign(&mut self, rhs: u8) {
        self.0 >>= rhs
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
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if self.0.none() {
            return None;
        }

        let idx = self.0 .0.trailing_zeros();
        let res = idx as u8;
        self.0 ^= BB::square(res);
        Some(res)
    }
}
