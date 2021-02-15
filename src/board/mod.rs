mod fen;
mod render;

bitflags! {
    struct ExtraState: u16{
        const BLACK_MOVE = 0b1;

        const WHITE_KING_CASTLE = 0b10;
        const WHITE_QUEEN_CASTLE = 0b100;

        const BLACK_KING_CASTLE = 0b1000;
        const BLACK_QUEEN_CASTLE = 0b10000;
    }
}

pub struct Board {
    white_king: u64,
    white_queens: u64,
    white_rooks: u64,
    white_bishops: u64,
    white_knights: u64,
    white_pawns: u64,

    black_king: u64,
    black_queens: u64,
    black_rooks: u64,
    black_bishops: u64,
    black_knights: u64,
    black_pawns: u64,

    en_passant: u64,
    state: ExtraState,
}

impl Board {
    pub fn empty() -> Self {
        Board {
            white_king: 0,
            white_queens: 0,
            white_rooks: 0,
            white_bishops: 0,
            white_knights: 0,
            white_pawns: 0,

            black_king: 0,
            black_queens: 0,
            black_rooks: 0,
            black_bishops: 0,
            black_knights: 0,
            black_pawns: 0,
            en_passant: 0,
            state: ExtraState::empty(),
        }
    }
}
