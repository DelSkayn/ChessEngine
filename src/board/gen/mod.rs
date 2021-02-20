use super::*;

pub struct MoveGenerator {
    knight_attacks: [BB; 64],
    king_attacks: [BB; 64],
    ray_attacks: [[BB; 64]; 8],
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Direction {
    NW = 0,
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
}

impl MoveGenerator {
    const DIRECTION_SHIFT: [i8; 8] = [7, 8, 9, 1, -7, -8, -9, -1];
    const DIRECTION_MASK: [BB; 8] = [
        BB(BB::FILE_A.0 | BB::RANK_8.0),
        BB::RANK_8,
        BB(BB::RANK_8.0 | BB::FILE_H.0),
        BB::FILE_H,
        BB(BB::FILE_H.0 | BB::RANK_1.0),
        BB::RANK_1,
        BB(BB::RANK_1.0 | BB::FILE_A.0),
        BB::FILE_A,
    ];

    pub fn new() -> Self {
        MoveGenerator {
            knight_attacks: MoveGenerator::gen_knight_attacks(),
            king_attacks: MoveGenerator::gen_king_attacks(),
            ray_attacks: MoveGenerator::gen_ray_attackes(),
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

    fn gen_ray_attackes() -> [[BB; 64]; 8] {
        let mut res = [[BB::empty(); 64]; 8];

        for d in 0..8 {
            let mask = Self::DIRECTION_MASK[d];
            let shift = Self::DIRECTION_SHIFT[d];
            for i in 0..64 {
                res[d][i] = BB::square(i as u8);
                for _ in 0..7 {
                    res[d][i] |= (res[d][i] & !mask).shift(shift);
                }
                res[d][i] &= !BB::square(i as u8)
            }
        }

        res
    }

    pub fn gen_moves(&self, b: &Board) -> Vec<Move> {
        let mut res = Vec::new();

        let player_pieces = b.pieces[Board::WHITE_KING]
            | b.pieces[Board::WHITE_QUEEN]
            | b.pieces[Board::WHITE_BISHOP]
            | b.pieces[Board::WHITE_KNIGHT]
            | b.pieces[Board::WHITE_ROOK]
            | b.pieces[Board::WHITE_PAWN];
        let opponent_pieces = b.pieces[Board::BLACK_KING]
            | b.pieces[Board::BLACK_QUEEN]
            | b.pieces[Board::BLACK_BISHOP]
            | b.pieces[Board::BLACK_KNIGHT]
            | b.pieces[Board::BLACK_ROOK]
            | b.pieces[Board::BLACK_PAWN];

        let all_pieces = player_pieces | opponent_pieces;

        let (attacked, pinned) = self.attacked(b, all_pieces, player_pieces);
        dbg!(attacked);
        dbg!(pinned);

        let pawns = b.pieces[Board::WHITE_PAWN];
        let free_pawns = pawns & !pinned;

        let left_pawn_attacks = ((free_pawns & !BB::FILE_A) << 9) & opponent_pieces;
        for p in left_pawn_attacks.iter() {
            if p >= 56 {
                for i in 1..5 {
                    res.push(Move {
                        promote: i,
                        from: p - 9,
                        to: p,
                        piece: Board::WHITE_PAWN as u8,
                    });
                }
            } else {
                res.push(Move {
                    promote: 0,
                    from: p - 9,
                    to: p,
                    piece: Board::WHITE_PAWN as u8,
                });
            }
        }

        let right_pawn_attacks = ((free_pawns & !BB::FILE_H) << 7) & opponent_pieces;
        for p in right_pawn_attacks.iter() {
            if p >= 56 {
                for i in 1..5 {
                    res.push(Move {
                        promote: i,
                        from: p - 7,
                        to: p,
                        piece: Board::WHITE_PAWN as u8,
                    });
                }
            } else {
                res.push(Move {
                    promote: 0,
                    from: p - 7,
                    to: p,
                    piece: Board::WHITE_PAWN as u8,
                });
            }
        }

        let pawn_moves = ((free_pawns & !pinned) << 8) & !all_pieces;
        for p in pawn_moves.iter() {
            if p >= 56 {
                for i in 1..5 {
                    res.push(Move {
                        promote: i,
                        from: p - 8,
                        to: p,
                        piece: Board::WHITE_PAWN as u8,
                    });
                }
            } else {
                res.push(Move {
                    promote: 0,
                    from: p - 8,
                    to: p,
                    piece: Board::WHITE_PAWN as u8,
                });
            }
        }

        let double_pawn_moves = ((pawn_moves & BB::RANK_3) << 8) & !all_pieces;
        for p in double_pawn_moves.iter() {
            res.push(Move {
                promote: 0,
                from: p - 16,
                to: p,
                piece: Board::WHITE_PAWN as u8,
            });
        }

        for p in (b.pieces[Board::WHITE_KNIGHT] & !pinned).iter() {
            let moves = self.knight_attacks[p as usize] & !player_pieces;
            for m in moves.iter() {
                res.push(Move {
                    promote: 0,
                    from: p,
                    to: m,
                    piece: Board::WHITE_KNIGHT as u8,
                })
            }
        }

        self.sliding_pieces(b, all_pieces, player_pieces, &mut res);

        for p in b.pieces[Board::WHITE_KING].iter() {
            let moves = self.king_attacks[p as usize] & !player_pieces & !attacked;
            for m in moves.iter() {
                res.push(Move {
                    promote: 0,
                    from: p,
                    to: m,
                    piece: Board::WHITE_KING as u8,
                })
            }
        }

        if (b.state & ExtraState::WHITE_KING_CASTLE).any() {
            let present = all_pieces & BB::WHITE_KING_CASTLE_MASK;
            let attacked = attacked & BB(6);
            if (present | attacked).none() {
                res.push(Move {
                    promote: 0,
                    from: 4,
                    to: 6,
                    piece: Board::WHITE_KING as u8,
                })
            }
        }

        if (b.state & ExtraState::WHITE_QUEEN_CASTLE).any() {
            let present = all_pieces & BB::WHITE_QUEEN_CASTLE_MASK;
            let attacked = attacked & BB(2);
            if (present | attacked).none() {
                res.push(Move {
                    promote: 0,
                    from: 4,
                    to: 2,
                    piece: Board::WHITE_KING as u8,
                })
            }
        }

        res
    }

    fn attacked(&self, b: &Board, occupied: BB, player: BB) -> (BB, BB) {
        let mut attacked = BB::empty();
        let mut pinned = BB::empty();
        let king = b.pieces[Board::WHITE_KING].0.trailing_zeros() as u8;

        for p in b.pieces[Board::BLACK_KING].iter() {
            attacked |= self.king_attacks[p as usize]
        }
        for p in b.pieces[Board::BLACK_KNIGHT].iter() {
            attacked |= self.knight_attacks[p as usize]
        }

        let pawn_attacks = (b.pieces[Board::BLACK_PAWN] & !BB::FILE_H) >> 7
            | (b.pieces[Board::BLACK_PAWN] & !BB::FILE_A) >> 9;
        dbg!(pawn_attacks);
        attacked |= pawn_attacks;

        for p in (b.pieces[Board::BLACK_BISHOP] | b.pieces[Board::BLACK_QUEEN]).iter() {
            let attack = self.ray_attacks_positive(p, occupied, Direction::NW);
            pinned |= attack & self.ray_attacks_negative(king, occupied, Direction::SE) & player;
            attacked |= attack;

            let attack = self.ray_attacks_positive(p, occupied, Direction::NE);
            pinned |= attack & self.ray_attacks_negative(king, occupied, Direction::SW) & player;
            attacked |= attack;

            let attack = self.ray_attacks_negative(p, occupied, Direction::SE);
            pinned |= attack & self.ray_attacks_positive(king, occupied, Direction::NW) & player;
            attacked |= attack;

            let attack = self.ray_attacks_negative(p, occupied, Direction::SW);
            pinned |= attack & self.ray_attacks_positive(king, occupied, Direction::NE) & player;
            attacked |= attack;
        }

        for p in (b.pieces[Board::BLACK_ROOK] | b.pieces[Board::BLACK_QUEEN]).iter() {
            let attack = self.ray_attacks_positive(p, occupied, Direction::N);
            pinned |= attack & self.ray_attacks_negative(king, occupied, Direction::S) & player;
            attacked |= attack;

            let attack = self.ray_attacks_positive(p, occupied, Direction::E);
            pinned |= attack & self.ray_attacks_negative(king, occupied, Direction::W) & player;
            attacked |= attack;

            let attack = self.ray_attacks_negative(p, occupied, Direction::S);
            pinned |= attack & self.ray_attacks_positive(king, occupied, Direction::N) & player;
            attacked |= attack;

            let attack = self.ray_attacks_negative(p, occupied, Direction::W);
            pinned |= attack & self.ray_attacks_positive(king, occupied, Direction::E) & player;
            attacked |= attack;
        }
        (attacked, pinned)
    }

    fn sliding_pieces(&self, b: &Board, occupied: BB, player: BB, buffer: &mut Vec<Move>) {
        for p in b.pieces[Board::WHITE_BISHOP].iter() {
            let mut attacks = self.ray_attacks_positive(p, occupied, Direction::NW)
                | self.ray_attacks_positive(p, occupied, Direction::NE)
                | self.ray_attacks_negative(p, occupied, Direction::SE)
                | self.ray_attacks_negative(p, occupied, Direction::SW);
            attacks &= !player;

            for m in attacks.iter() {
                buffer.push(Move {
                    promote: 0,
                    from: p,
                    to: m,
                    piece: Board::WHITE_BISHOP as u8,
                })
            }
        }

        for p in b.pieces[Board::WHITE_ROOK].iter() {
            let mut attacks = self.ray_attacks_positive(p, occupied, Direction::N)
                | self.ray_attacks_positive(p, occupied, Direction::E)
                | self.ray_attacks_negative(p, occupied, Direction::S)
                | self.ray_attacks_negative(p, occupied, Direction::W);
            attacks &= !player;

            for m in attacks.iter() {
                buffer.push(Move {
                    promote: 0,
                    from: p,
                    to: m,
                    piece: Board::WHITE_ROOK as u8,
                })
            }
        }

        for p in b.pieces[Board::WHITE_QUEEN].iter() {
            let mut attacks = self.ray_attacks_positive(p, occupied, Direction::NW)
                | self.ray_attacks_positive(p, occupied, Direction::N)
                | self.ray_attacks_positive(p, occupied, Direction::NE)
                | self.ray_attacks_positive(p, occupied, Direction::E)
                | self.ray_attacks_negative(p, occupied, Direction::SE)
                | self.ray_attacks_negative(p, occupied, Direction::S)
                | self.ray_attacks_negative(p, occupied, Direction::SW)
                | self.ray_attacks_negative(p, occupied, Direction::W);

            attacks &= !player;

            for m in attacks.iter() {
                buffer.push(Move {
                    promote: 0,
                    from: p,
                    to: m,
                    piece: Board::WHITE_QUEEN as u8,
                })
            }
        }
    }

    fn ray_attacks_positive(&self, square: u8, occupied: BB, direction: Direction) -> BB {
        let attack = self.ray_attacks[direction as usize][square as usize];
        let blockers = attack & occupied;
        let block_square = (blockers | BB::B8).0.trailing_zeros();
        attack ^ self.ray_attacks[direction as usize][block_square as usize]
    }

    fn ray_attacks_negative(&self, square: u8, occupied: BB, direction: Direction) -> BB {
        let attack = self.ray_attacks[direction as usize][square as usize];
        let blockers = attack & occupied;
        let block_square = 63 - (blockers | BB::A1).0.leading_zeros();
        attack ^ self.ray_attacks[direction as usize][block_square as usize]
    }
}
