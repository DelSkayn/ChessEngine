use super::*;

pub struct MoveGenerator {
    knight_attacks: [BB; 64],
    king_attacks: [BB; 64],
}

impl MoveGenerator {
    pub fn new() -> Self {
        MoveGenerator {
            knight_attacks: MoveGenerator::gen_knight_attacks(),
            king_attacks: MoveGenerator::gen_king_attacks(),
        }
    }

    fn gen_knight_attacks() -> [BB; 64] {
        let mut res = [BB::empty(); 64];
        for i in 0..64 {
            let position = BB::square(i);
            let east = (position & !BB::FILE_A) >> 1;
            let west = (position & !BB::FILE_H) << 1;
            let ew = east | west;
            let north = (ew & !(BB::RANK_7 | BB::RANK_8)) << 16;
            let south = (ew & !(BB::RANK_1 | BB::RANK_2)) >> 16;
            let east = (east & !BB::FILE_A) >> 1;
            let west = (west & !BB::FILE_H) << 1;
            let ew = east | west;
            let north = ((ew & !BB::RANK_8) << 8) | north;
            let south = ((ew & !BB::RANK_1) >> 8) | south;
            res[i as usize] = north | south;
        }
        res
    }

    fn gen_king_attacks() -> [BB; 64] {
        let mut res = [BB::empty(); 64];
        for i in 0..64 {
            let position = BB::square(i);
            let east = (position & !BB::FILE_A) >> 1;
            let west = (position & !BB::FILE_H) << 1;
            let ewp = east | west | position;

            let north = (ewp & !BB::RANK_8) << 8;
            let south = (ewp & !BB::RANK_1) >> 8;
            res[i as usize] = north | south | east | west;
        }
        res
    }

    pub fn gen_moves(&self, b: &Board) -> Vec<Move> {
        let mut res = Vec::new();

        let player_pieces = b.pieces[Board::WHITE_KING as usize]
            | b.pieces[Board::WHITE_QUEEN as usize]
            | b.pieces[Board::WHITE_BISHOP as usize]
            | b.pieces[Board::WHITE_KNIGHT as usize]
            | b.pieces[Board::WHITE_ROOK as usize]
            | b.pieces[Board::WHITE_PAWN as usize];
        let opponent_pieces = b.pieces[Board::BLACK_KING as usize]
            | b.pieces[Board::BLACK_QUEEN as usize]
            | b.pieces[Board::BLACK_BISHOP as usize]
            | b.pieces[Board::BLACK_KNIGHT as usize]
            | b.pieces[Board::BLACK_ROOK as usize]
            | b.pieces[Board::BLACK_PAWN as usize];

        let all_pieces = player_pieces | opponent_pieces;

        let pawns = b.pieces[Board::WHITE_PAWN as usize];

        let left_pawn_attacks = ((pawns & !BB::FILE_A) << 9) & opponent_pieces;
        for p in left_pawn_attacks.iter() {
            res.push(Move {
                from: p - 9,
                to: p,
                piece: Board::WHITE_PAWN,
            });
        }

        let right_pawn_attacks = ((pawns & !BB::FILE_H) << 7) & opponent_pieces;
        for p in right_pawn_attacks.iter() {
            res.push(Move {
                from: p + 7,
                to: p,
                piece: Board::WHITE_PAWN,
            });
        }

        let pawn_moves = (pawns << 8) & !all_pieces;
        for p in pawn_moves.iter() {
            res.push(Move {
                from: p - 8,
                to: p,
                piece: Board::WHITE_PAWN,
            });
        }

        let double_pawn_moves = ((pawn_moves & BB::RANK_3) << 8) & !all_pieces;
        for p in double_pawn_moves.iter() {
            res.push(Move {
                from: p - 16,
                to: p,
                piece: Board::WHITE_PAWN,
            });
        }

        for p in b.pieces[Board::WHITE_KNIGHT as usize].iter() {
            let moves = self.knight_attacks[p as usize] & !player_pieces;
            for m in moves.iter() {
                res.push(Move {
                    from: p,
                    to: m,
                    piece: Board::WHITE_KNIGHT,
                })
            }
        }

        for p in b.pieces[Board::WHITE_KING as usize].iter() {
            let moves = self.king_attacks[p as usize] & !player_pieces;
            for m in moves.iter() {
                res.push(Move {
                    from: p,
                    to: m,
                    piece: Board::WHITE_KING,
                })
            }
        }

        res
    }
}
