use crate::search;

use super::AlphaBeta;
use chess_core::{gen::PositionInfo, util::BoardArray, Piece, Player};

pub const PAWN_VALUE: i32 = 100;
pub const KNIGHT_VALUE: i32 = 320;
pub const BISHOP_VALUE: i32 = 325;
pub const ROOK_VALUE: i32 = 500;
pub const QUEEN_VALUE: i32 = 975;

impl<C> AlphaBeta<C> {
    const FULL_PIECE_VALUE: i32 =
        QUEEN_VALUE + BISHOP_VALUE * 2 + KNIGHT_VALUE * 2 + ROOK_VALUE * 2;
    const PIECE_VALUE: [i32; 12] = [
        0,
        QUEEN_VALUE,
        BISHOP_VALUE,
        KNIGHT_VALUE,
        ROOK_VALUE,
        PAWN_VALUE,
        0,
        QUEEN_VALUE,
        BISHOP_VALUE,
        KNIGHT_VALUE,
        ROOK_VALUE,
        PAWN_VALUE,
    ];

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

    const ROOK_TABLE: BoardArray<i32> = BoardArray::new_array([
        0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, 10, 10, 10, 10, 5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0,
        0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0,
        -5, 0, 0, 0, 5, 5, 0, 0, 0,
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
            return color * search::CHECKMATE_SCORE;
        }

        let white_piece_value: i32 = Piece::WhiteQueen
            .to(Piece::WhiteRook)
            .map(|x| b.pieces[x].count() as i32 * Self::PIECE_VALUE[x as usize])
            .sum();

        let black_piece_value: i32 = Piece::BlackQueen
            .to(Piece::BlackRook)
            .map(|x| b.pieces[x].count() as i32 * Self::PIECE_VALUE[x as usize])
            .sum();

        let white_earlygame = white_piece_value as f32 / Self::FULL_PIECE_VALUE as f32;
        let black_earlygame = black_piece_value as f32 / Self::FULL_PIECE_VALUE as f32;

        let white_king_sq = b.pieces[Piece::WhiteKing].first_piece();
        let black_king_sq = b.pieces[Piece::BlackKing].first_piece();

        let white_king_score = (Self::KING_TABLE[white_king_sq.flip()] as f32 * white_earlygame
            + Self::KING_END_TABLE[white_king_sq.flip()] as f32 * (1.0 - white_earlygame))
            as i32;
        let black_king_score = (Self::KING_TABLE[black_king_sq] as f32 * black_earlygame
            + Self::KING_END_TABLE[black_king_sq] as f32 * (1.0 - black_earlygame))
            as i32;

        let mut piece_value = white_king_score - black_king_score + white_piece_value
            - black_piece_value
            + (b.pieces[Piece::WhitePawn].count() as i32
                - b.pieces[Piece::BlackPawn].count() as i32)
                * PAWN_VALUE;

        for p in b.pieces[Piece::WhiteBishop].iter() {
            piece_value += Self::BISHOP_TABLE[p.flip()]
        }
        for p in b.pieces[Piece::WhiteKnight].iter() {
            piece_value += Self::KNIGHT_TABLE[p.flip()]
        }
        for p in b.pieces[Piece::WhiteRook].iter() {
            piece_value += Self::ROOK_TABLE[p.flip()]
        }
        for p in b.pieces[Piece::WhitePawn].iter() {
            piece_value += Self::PAWN_TABLE[p.flip()]
        }

        for p in b.pieces[Piece::BlackBishop].iter() {
            piece_value -= Self::BISHOP_TABLE[p]
        }
        for p in b.pieces[Piece::BlackKnight].iter() {
            piece_value -= Self::KNIGHT_TABLE[p]
        }
        for p in b.pieces[Piece::BlackRook].iter() {
            piece_value -= Self::ROOK_TABLE[p]
        }
        for p in b.pieces[Piece::BlackPawn].iter() {
            piece_value -= Self::PAWN_TABLE[p]
        }

        piece_value
    }
}
