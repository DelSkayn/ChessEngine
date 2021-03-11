use std::{
    fmt::{self, Debug},
    ops::{Add, Sub},
};

#[derive(Eq, PartialEq, Clone, Copy)]
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

    pub fn new(v: u8) -> Self {
        debug_assert!(v < 64);
        Square(v)
    }

    pub fn from_name(name: &str) -> Option<Self> {
        let mut chars = name.chars();
        let file = chars.next()?;
        let rank = chars.next()?;
        if ('a'..='h').contains(&file) || ('A'..='H').contains(&file) {
            if !('1'..='8').contains(&rank) {
                return None;
            }
            let file = file.to_ascii_lowercase() as u8 - 'a' as u8;
            let rank = rank as u8 - '1' as u8;
            return Some(Self::from_file_rank(file, rank));
        }
        None
    }

    pub fn from_file_rank(file: u8, rank: u8) -> Self {
        let res = (file & 7) | ((rank & 7) << 3);
        debug_assert!(res < 64);
        Square(res)
    }

    pub fn to_file_rank(self) -> (u8, u8) {
        (self.file(), self.rank())
    }

    pub const fn get(self) -> u8 {
        self.0
    }

    pub fn file(self) -> u8 {
        self.0 & 7
    }

    pub fn rank(self) -> u8 {
        self.0 >> 3
    }

    pub fn flip(self) -> Self {
        Square(63 - self.0)
    }
}

impl Add<u8> for Square {
    type Output = Self;

    fn add(mut self, rhs: u8) -> Self::Output {
        self.0 += rhs;
        debug_assert!(self.0 < 64);
        self
    }
}

impl Sub<u8> for Square {
    type Output = Self;

    fn sub(mut self, rhs: u8) -> Self::Output {
        self.0 -= rhs;
        debug_assert!(self.0 < 64);
        self
    }
}

impl Add<i8> for Square {
    type Output = Self;

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

    fn sub(mut self, rhs: i8) -> Self::Output {
        let tmp = self.0 as i8 - rhs;
        debug_assert!(tmp > 0);
        self.0 = tmp as u8;
        debug_assert!(self.0 < 64);
        self
    }
}

impl Debug for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let file = self.0 % 8;
        let rank = self.0 / 8;
        let file_name = ('a' as u8 + file) as char;
        write!(f, "{}{}", file_name, rank + 1)
    }
}
