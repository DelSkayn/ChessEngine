use std::fmt::{self, Debug};

#[derive(Eq, PartialEq, Clone, Copy)]
pub struct ExtraState {
    pub black_turn: bool,
    pub castle: u8,
    pub en_passant: u8,
}

impl ExtraState {
    pub const WHITE_KING_CASTLE: u8 = 0b0001;
    pub const WHITE_QUEEN_CASTLE: u8 = 0b0010;
    pub const BLACK_KING_CASTLE: u8 = 0b0100;
    pub const BLACK_QUEEN_CASTLE: u8 = 0b1000;

    pub const fn empty() -> Self {
        ExtraState {
            black_turn: false,
            castle: 0,
            en_passant: u8::MAX,
        }
    }

    pub fn flip(mut self) -> Self {
        self.castle = ((0b11) & self.castle) << 2 | ((0b11 << 2) & self.castle) >> 2;
        self.en_passant = 7 - self.en_passant;
        self.black_turn = !self.black_turn;
        self
    }
}

impl Debug for ExtraState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExtraState")
            .field("black_turn", &self.black_turn)
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
