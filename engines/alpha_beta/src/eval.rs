use common::{BoardArray, Piece};

use super::AlphaBeta;

impl AlphaBeta {
    pub const PAWN_VALUE: i32 = 100;
    pub const KNIGHT_VALUE: i32 = 320;
    pub const BISHOP_VALUE: i32 = 325;
    pub const ROOK_VALUE: i32 = 500;
    pub const QUEEN_VALUE: i32 = 975;

    const FULL_PIECE_VALUE: i32 =
        Self::QUEEN_VALUE + Self::BISHOP_VALUE * 2 + Self::KNIGHT_VALUE * 2 + Self::ROOK_VALUE * 2;

    const PAWN_TABLE: BoardArray<i8> = BoardArray::new_array([
        0, 0, 0, 0, 0, 0, 0, 0, 50, 50, 50, 50, 50, 50, 50, 50, 10, 10, 20, 30, 30, 20, 10, 10, 5,
        5, 10, 27, 27, 10, 5, 5, 0, 0, 0, 25, 25, 0, 0, 0, 5, -5, -10, 0, 0, -10, -5, 5, 5, 10, 10,
        -25, -25, 10, 10, 5, 0, 0, 0, 0, 0, 0, 0, 0,
    ]);

    const KNIGHT_TABLE: BoardArray<i8> = BoardArray::new_array([
        -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 0, 0, 0, -20, -40, -30, 0, 10, 15, 15,
        10, 0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5, 10, 15,
        15, 10, 5, -30, -40, -20, 0, 5, 5, 0, -20, -40, -50, -40, -20, -30, -30, -20, -40, -50,
    ]);

    const BISHOP_TABLE: BoardArray<i8> = BoardArray::new_array([
        -20, -10, -10, -10, -10, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 10, 10, 5,
        0, -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 10, 10, 10, 10,
        10, 10, -10, -10, 5, 0, 0, 0, 0, 5, -10, -20, -10, -40, -10, -10, -40, -10, -20,
    ]);

    const ROOK_TABLE: BoardArray<i8> = BoardArray::new_array([
        0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, 10, 10, 10, 10, 5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0,
        0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0,
        -5, 0, 0, 0, 5, 5, 0, 0, 0,
    ]);

    const KING_TABLE: BoardArray<i8> = BoardArray::new_array([
        -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40,
        -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -20, -30, -30, -40,
        -40, -30, -30, -20, -10, -20, -20, -20, -20, -20, -20, -10, 20, 20, 0, 0, 0, 0, 20, 20, 20,
        30, 10, 0, 0, 10, 30, 20,
    ]);

    const KING_END_TABLE: BoardArray<i8> = BoardArray::new_array([
        -50, -40, -30, -20, -20, -30, -40, -50, -30, -20, -10, 0, 0, -10, -20, -30, -30, -10, 20,
        30, 30, 20, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30, -10, 30, 40, 40, 30, -10,
        -30, -30, -10, 20, 30, 30, 20, -10, -30, -30, -30, 0, 0, 0, 0, -30, -30, -50, -30, -30,
        -30, -30, -30, -30, -50,
    ]);

    pub fn eval(&mut self) -> i32 {
        let white_material_score = self.board.pieces[Piece::WhiteBishop].count() as i32
            * Self::BISHOP_VALUE
            + self.board.pieces[Piece::WhiteKnight].count() as i32 * Self::KNIGHT_VALUE
            + self.board.pieces[Piece::WhiteRook].count() as i32 * Self::ROOK_VALUE
            + self.board.pieces[Piece::WhiteQueen].count() as i32 * Self::QUEEN_VALUE;

        let black_material_score = self.board.pieces[Piece::BlackBishop].count() as i32
            * Self::BISHOP_VALUE
            + self.board.pieces[Piece::BlackKnight].count() as i32 * Self::KNIGHT_VALUE
            + self.board.pieces[Piece::BlackRook].count() as i32 * Self::ROOK_VALUE
            + self.board.pieces[Piece::BlackQueen].count() as i32 * Self::QUEEN_VALUE;

        // interpolate between two tables depending on the opposite players material score.
        let white_king_square = self.board.pieces[Piece::WhiteKing].last_piece();
        let max = Self::FULL_PIECE_VALUE.max(black_material_score);
        let white_king_score = (Self::KING_TABLE[white_king_square] as i32 * black_material_score
            + Self::KING_END_TABLE[white_king_square] as i32 * (max - black_material_score))
            / max;

        let black_king_square = self.board.pieces[Piece::BlackKing].last_piece().flip();
        let max = Self::FULL_PIECE_VALUE.max(white_material_score);
        let black_king_score = (Self::KING_TABLE[black_king_square] as i32 * white_material_score
            + Self::KING_END_TABLE[black_king_square] as i32 * (max - white_material_score))
            / max;

        let mut score = self.board.pieces[Piece::WhitePawn].count() as i32 * Self::PAWN_VALUE
            - self.board.pieces[Piece::BlackPawn].count() as i32 * Self::PAWN_VALUE
            + white_material_score
            + white_king_score
            - black_material_score
            - black_king_score;

        for rook in self.board.pieces[Piece::WhitePawn].iter() {
            score += Self::PAWN_TABLE[rook] as i32;
        }
        for rook in self.board.pieces[Piece::BlackPawn].iter() {
            score -= Self::PAWN_TABLE[rook.flip()] as i32;
        }

        for rook in self.board.pieces[Piece::WhiteBishop].iter() {
            score += Self::BISHOP_TABLE[rook] as i32;
        }
        for rook in self.board.pieces[Piece::BlackBishop].iter() {
            score -= Self::BISHOP_TABLE[rook.flip()] as i32;
        }

        for rook in self.board.pieces[Piece::WhiteKnight].iter() {
            score += Self::KNIGHT_TABLE[rook] as i32;
        }
        for rook in self.board.pieces[Piece::BlackKnight].iter() {
            score -= Self::KNIGHT_TABLE[rook.flip()] as i32;
        }

        for rook in self.board.pieces[Piece::WhiteRook].iter() {
            score += Self::ROOK_TABLE[rook] as i32;
        }
        for rook in self.board.pieces[Piece::BlackRook].iter() {
            score -= Self::ROOK_TABLE[rook.flip()] as i32;
        }

        score
    }
}
