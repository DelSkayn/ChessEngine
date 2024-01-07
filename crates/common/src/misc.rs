use std::ops;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WinCause {
    Timeout,
    Mate,
    Disconnect,
    Other,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DrawCause {
    Stalemate,
    Timeout,
    FiftyMove,
    Repetition,
    Agreement,
    Material,
    Disconnect,
    Other,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Outcome {
    /// The game was won by a player
    Won { by: Player, cause: WinCause },
    /// THe game was drawn
    Drawn(DrawCause),
    /// No outcome was determined.
    None,
}

/// Enum representing a player.
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Player {
    White,
    Black,
}

impl ops::Not for Player {
    type Output = Self;

    fn not(self) -> Self::Output {
        self.flip()
    }
}

impl Player {
    /// Returns the other player
    #[inline]
    pub fn flip(self) -> Self {
        unsafe { Self::from_u8_unchecked(self as u8 ^ 0b1) }
    }

    /// Create player from u8 representation
    pub fn from_u8(v: u8) -> Self {
        assert!(v < 2);
        unsafe { Self::from_u8_unchecked(v) }
    }

    /// Create player from u8 representation without checking if value is valid.
    ///
    /// # Safety
    /// Caller must ensure that the give representation is valid.
    #[inline]
    pub unsafe fn from_u8_unchecked(v: u8) -> Self {
        debug_assert!(v < 2);
        std::mem::transmute(v)
    }
}

/// A direction on the board with the side with the black pieces being north.
#[repr(i8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Direction {
    NW = 7,
    N = 8,
    NE = 9,
    E = 1,
    SE = -7,
    S = -8,
    SW = -9,
    W = -1,
}

impl Direction {
    /// Creates a direction from u8
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Direction::NW),
            1 => Some(Direction::N),
            2 => Some(Direction::NE),
            3 => Some(Direction::E),
            4 => Some(Direction::SE),
            5 => Some(Direction::S),
            6 => Some(Direction::SW),
            7 => Some(Direction::W),
            _ => None,
        }
    }

    /// Returns the board square index offset of the direction.
    #[inline(always)]
    pub const fn as_offset(self) -> i8 {
        self as i8
    }
}
