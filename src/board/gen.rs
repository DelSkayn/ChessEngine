use super::*;

impl Board {
    pub fn gen_moves(&self) -> Vec<Move> {
        let mut res = Vec::new();

        let player_pieces = self.pieces[Board::WHITE_QUEEN as usize]
            | self.pieces[Board::WHITE_BISHOP as usize]
            | self.pieces[Board::WHITE_KNIGHT as usize]
            | self.pieces[Board::WHITE_ROOK as usize]
            | self.pieces[Board::WHITE_PAWN as usize];
        let opponent_pieces = self.pieces[Board::BLACK_QUEEN as usize]
            | self.pieces[Board::BLACK_BISHOP as usize]
            | self.pieces[Board::BLACK_KNIGHT as usize]
            | self.pieces[Board::BLACK_ROOK as usize]
            | self.pieces[Board::BLACK_PAWN as usize];

        let all_pieces = player_pieces | opponent_pieces;

        let pawns = self.pieces[Board::WHITE_PAWN as usize];

        let left_pawn_attacks = ((pawns & !BB::FILE_A) >> 9) & opponent_pieces;
        for p in left_pawn_attacks.iter() {
            res.push(Move {
                from: p + 9,
                to: p,
                piece: Board::WHITE_PAWN,
            });
        }

        let right_pawn_attacks = ((pawns & !BB::FILE_H) >> 7) & opponent_pieces;
        for p in right_pawn_attacks.iter() {
            res.push(Move {
                from: p + 9,
                to: p,
                piece: Board::WHITE_PAWN,
            });
        }

        let pawn_moves = (pawns >> 8) & !all_pieces;
        for p in pawn_moves.iter() {
            res.push(Move {
                from: p + 8,
                to: p,
                piece: Board::WHITE_PAWN,
            });
        }

        let double_pawn_moves = ((pawn_moves & BB::RANK_3) >> 8) & !all_pieces;
        for p in double_pawn_moves.iter() {
            res.push(Move {
                from: p + 16,
                to: p,
                piece: Board::WHITE_PAWN,
            });
        }

        res
    }
}
