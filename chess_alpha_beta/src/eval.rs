use super::AlphaBeta;
use chess_core::{gen::PositionInfo, util::BoardArray, Piece, Player};

impl AlphaBeta {
    pub const PAWN_VALUE: i32 = 100;
    pub const KNIGHT_VALUE: i32 = 320;
    pub const BISHOP_VALUE: i32 = 325;
    pub const ROOK_VALUE: i32 = 500;
    pub const QUEEN_VALUE: i32 = 975;

    const PAWN_TABLE: BoardArray<i32> = BoardArray::new_array([
        0, 0, 0, 0, 0, 0, 0, 0, 50, 50, 50, 50, 50, 50, 50, 50, 10, 10, 20, 30, 30, 20, 10, 10, 5,
        5, 10, 27, 27, 10, 5, 5, 0, 0, 0, 25, 25, 0, 0, 0, 5, -5, -10, 0, 0, -10, -5, 5, 5, 10, 10,
        -25, -25, 10, 10, 5, 0, 0, 0, 0, 0, 0, 0, 0,
    ]);

    const KNIGHT_TABLE: BoardArray<i32> = BoardArray::new_array([
        -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 0, 0, 0, -20, -40, -30, 0, 10, 15, 15,
        10, 0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5, 10, 15,
        15, 10, 5, -30, -40, -20, 0, 5, 5, 0, -20, -40, -50, -40, -20, -30, -30, -20, -40, -50,
    ]);

    const BISHOP_TABLE: BoardArray<i32> = BoardArray::new_array([
        -20, -10, -10, -10, -10, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 10, 10, 5,
        0, -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 10, 10, 10, 10,
        10, 10, -10, -10, 5, 0, 0, 0, 0, 5, -10, -20, -10, -40, -10, -10, -40, -10, -20,
    ]);

    const KING_TABLE: BoardArray<i32> = BoardArray::new_array([
        -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40,
        -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -20, -30, -30, -40,
        -40, -30, -30, -20, -10, -20, -20, -20, -20, -20, -20, -10, 20, 20, 0, 0, 0, 0, 20, 20, 20,
        30, 10, 0, 0, 10, 30, 20,
    ]);

    const KING_END_TABLE: BoardArray<i32> = BoardArray::new_array([
        -50, -40, -30, -20, -20, -30, -40, -50, -30, -20, -10, 0, 0, -10, -20, -30, -30, -10, 20,
        30, 30, 20, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30, -10, 30, 40, 40, 30, -10,
        -30, -30, -10, 20, 30, 30, 20, -10, -30, -30, -30, 0, 0, 0, 0, -30, -30, -50, -30, -30,
        -30, -30, -30, -30, -50,
    ]);

    pub fn eval_board(&mut self, info: &PositionInfo) -> i32 {
        let b = &self.board;
        self.nodes += 1;

        if self.gen.check_mate(b, info) {
            let color = match b.state.player {
                Player::White => -1,
                Player::Black => 1,
            };
            return color * Self::CHECKMATE_SCORE;
        }

        let (white_occ, black_occ) = if b.state.player == Player::White {
            (info.my, info.their)
        } else {
            (info.their, info.my)
        };

        let white_pieces = (white_occ & !b.pieces[Piece::WhitePawn]).count() - 1;
        let black_pieces = (black_occ & !b.pieces[Piece::BlackPawn]).count() - 1;

        let mut piece_value = (b.pieces[Piece::WhiteQueen].count() as i32
            - b.pieces[Piece::BlackQueen].count() as i32)
            * Self::QUEEN_VALUE
            + (b.pieces[Piece::WhiteRook].count() as i32
                - b.pieces[Piece::BlackRook].count() as i32)
                * Self::ROOK_VALUE
            + (b.pieces[Piece::WhiteBishop].count() as i32
                - b.pieces[Piece::BlackBishop].count() as i32)
                * Self::BISHOP_VALUE
            + (b.pieces[Piece::WhiteKnight].count() as i32
                - b.pieces[Piece::BlackKnight].count() as i32)
                * Self::KNIGHT_VALUE
            + (b.pieces[Piece::WhitePawn].count() as i32
                - b.pieces[Piece::BlackPawn].count() as i32)
                * Self::PAWN_VALUE;

        //TODO do based on score instead of pieces
        for p in b.pieces[Piece::WhiteKing].iter() {
            let p = p.flip();
            piece_value += (Self::KING_TABLE[p] * white_pieces as i32
                + Self::KING_END_TABLE[p] * (7 - white_pieces as i32))
                / 7;
        }
        for p in b.pieces[Piece::WhiteBishop].iter() {
            piece_value += Self::BISHOP_TABLE[p.flip()]
        }
        for p in b.pieces[Piece::WhiteKnight].iter() {
            piece_value += Self::KNIGHT_TABLE[p.flip()]
        }
        for p in b.pieces[Piece::WhitePawn].iter() {
            piece_value += Self::PAWN_TABLE[p.flip()]
        }

        //TODO do based on score instead of pieces
        for p in b.pieces[Piece::BlackKing].iter() {
            piece_value -= (Self::KING_TABLE[p] * black_pieces as i32
                + Self::KING_END_TABLE[p] * (7 - black_pieces as i32))
                / 7;
        }
        for p in b.pieces[Piece::BlackBishop].iter() {
            piece_value -= Self::BISHOP_TABLE[p]
        }
        for p in b.pieces[Piece::BlackKnight].iter() {
            piece_value -= Self::KNIGHT_TABLE[p]
        }
        for p in b.pieces[Piece::BlackPawn].iter() {
            piece_value -= Self::PAWN_TABLE[p]
        }

        piece_value
    }
}
