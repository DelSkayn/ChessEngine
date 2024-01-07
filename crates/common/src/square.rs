use std::{
    fmt::{self, Display},
    ops::{Add, Sub},
};

use crate::BB;

/// A single square of the board
///
/// Index starts with the left bottom side from white's view being index 0
/// and the top right square being 63
/// Note that creating a square outside of the range 0..=63 can result in undefined behaviour.
#[repr(transparent)]
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Square(u8);

impl Square {
    pub const A1: Square = Square(0);
    pub const B1: Square = Square(1);
    pub const C1: Square = Square(2);
    pub const D1: Square = Square(3);
    pub const E1: Square = Square(4);
    pub const F1: Square = Square(5);
    pub const G1: Square = Square(6);
    pub const H1: Square = Square(7);

    pub const A8: Square = Square(56);
    pub const B8: Square = Square(57);
    pub const C8: Square = Square(58);
    pub const D8: Square = Square(59);
    pub const E8: Square = Square(60);
    pub const F8: Square = Square(61);
    pub const G8: Square = Square(62);
    pub const H8: Square = Square(63);

    /// Create a square from board index.
    #[inline]
    pub const fn new(v: u8) -> Self {
        assert!(v < 64);
        Square(v)
    }

    /// Create a square from board index.
    ///
    /// # Safety
    ///
    /// Caller muste ensure that the value is no larger then 63
    #[inline]
    pub const unsafe fn new_unchecked(v: u8) -> Self {
        debug_assert!(v < 64);
        Square(v)
    }

    /// Create a square from its name
    ///
    ///```rust
    /// # use chess_common::Square;
    ///let s = Square::from_name("a3");
    ///```
    pub fn from_name(name: &str) -> Option<Self> {
        let mut chars = name.chars();
        let file = chars.next()?;
        let rank = chars.next()?;
        if ('a'..='h').contains(&file) || ('A'..='H').contains(&file) {
            if !('1'..='8').contains(&rank) {
                return None;
            }
            let file = file.to_ascii_lowercase() as u8 - b'a';
            let rank = rank as u8 - b'1';
            return Some(Self::from_file_rank(file, rank));
        }
        None
    }

    /// Create a square from file and rank indecies
    pub fn from_file_rank(file: u8, rank: u8) -> Self {
        let res = (file & 7) | ((rank & 7) << 3);
        debug_assert!(res < 64);
        Square(res)
    }

    /// Returns the file and rank of the square
    pub const fn to_file_rank(self) -> (u8, u8) {
        (self.file(), self.rank())
    }

    /// Returns the index of the square
    #[inline]
    pub const fn get(self) -> u8 {
        self.0
    }

    /// Returns the file index of the square
    #[inline]
    pub const fn file(self) -> u8 {
        self.0 & 7
    }

    /// Returns the file rank of the square
    #[inline]
    pub const fn rank(self) -> u8 {
        self.0 >> 3
    }

    /// Flips the square so it's position is mirrored
    #[inline]
    pub const fn flip(self) -> Self {
        Square(63 - self.0)
    }

    #[inline]
    pub const fn to_bb(self) -> BB {
        BB::square(self)
    }
}

impl Add<u8> for Square {
    type Output = Self;

    #[inline]
    fn add(mut self, rhs: u8) -> Self::Output {
        self.0 += rhs;
        debug_assert!(self.0 < 64);
        self
    }
}

impl Sub<u8> for Square {
    type Output = Self;

    #[inline]
    fn sub(mut self, rhs: u8) -> Self::Output {
        self.0 -= rhs;
        debug_assert!(self.0 < 64);
        self
    }
}

impl Add<i8> for Square {
    type Output = Self;

    #[inline]
    fn add(mut self, rhs: i8) -> Self::Output {
        let tmp = self.0 as i8 + rhs;
        debug_assert!(tmp > 0);
        self.0 = tmp as u8;
        debug_assert!(self.0 < 64);
        self
    }
}

impl Sub<i8> for Square {
    type Output = Self;

    #[inline]
    fn sub(mut self, rhs: i8) -> Self::Output {
        let tmp = self.0 as i8 - rhs;
        debug_assert!(tmp > 0);
        self.0 = tmp as u8;
        debug_assert!(self.0 < 64);
        self
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let file = self.0 % 8;
        let rank = self.0 / 8;
        let file_name = (b'a' + file) as char;
        write!(f, "{}{}", file_name, rank + 1)
    }
}
