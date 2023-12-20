use crate::Player;
use std::fmt::{self, Debug};

/// Extra state containing information about a position other then the place of all the pieces
#[derive(Eq, PartialEq, Clone, Copy)]
pub struct ExtraState {
    /// The player whose turn it is.
    pub player: Player,
    /// Castling rights.
    pub castle: u8,
    /// The enpassant rank, 0-7 for rank, 8 for no enpassant took place.
    pub en_passant: u8,
    /// The amount of move since a irrevokable move has happend.
    pub move_clock: u8,
}

impl ExtraState {
    pub const WHITE_KING_CASTLE: u8 = 0b0001;
    pub const WHITE_QUEEN_CASTLE: u8 = 0b0010;
    pub const BLACK_KING_CASTLE: u8 = 0b0100;
    pub const BLACK_QUEEN_CASTLE: u8 = 0b1000;

    pub const INVALID_ENPASSANT: u8 = 8;

    pub const fn empty() -> Self {
        ExtraState {
            player: Player::White,
            castle: 0,
            en_passant: Self::INVALID_ENPASSANT,
            move_clock: 0,
        }
    }

    pub fn flip(mut self) -> Self {
        self.castle = ((0b11) & self.castle) << 2 | ((0b11 << 2) & self.castle) >> 2;
        self.en_passant = 7 - self.en_passant;
        self.player = self.player.flip();
        self
    }
}

impl Debug for ExtraState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExtraState")
            .field("player", &self.player)
            .field(
                "white_king_castle",
                &((self.castle & ExtraState::WHITE_KING_CASTLE) != 0),
            )
            .field(
                "white_queen_castle",
                &((self.castle & ExtraState::WHITE_QUEEN_CASTLE) != 0),
            )
            .field(
                "black_king_castle",
                &((self.castle & ExtraState::BLACK_KING_CASTLE) != 0),
            )
            .field(
                "black_queen_castle",
                &((self.castle & ExtraState::BLACK_QUEEN_CASTLE) != 0),
            )
            .field("en_passant", &self.en_passant)
            .finish()
    }
}
