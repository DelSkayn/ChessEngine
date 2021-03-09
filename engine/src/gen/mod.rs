use super::*;
mod util;

use util::{BoardArray, DirectionArray};
mod fill_7;

pub struct MoveGenerator {
    knight_attacks: BoardArray<BB>,
    king_attacks: BoardArray<BB>,
    ray_attacks: DirectionArray<BoardArray<BB>>,
    between: BoardArray<BoardArray<BB>>,
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

impl Direction {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Direction::NW,
            1 => Direction::N,
            2 => Direction::NE,
            3 => Direction::E,
            4 => Direction::SE,
            5 => Direction::S,
            6 => Direction::SW,
            7 => Direction::W,
            _ => panic!(),
        }
    }
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
            between: MoveGenerator::gen_between(),
        }
    }

    fn gen_knight_attacks() -> BoardArray<BB> {
        let mut res = BoardArray::new(BB::empty());
        for i in 0..64 {
            let position = BB::square(Square::new(i));
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
            res[Square::new(i)] = north | south;
        }
        res
    }

    fn gen_king_attacks() -> BoardArray<BB> {
        let mut res = BoardArray::new(BB::empty());
        for i in 0..64 {
            let position = BB::square(Square::new(i));
            let east = (position & !BB::FILE_A) >> 1;
            let west = (position & !BB::FILE_H) << 1;
            let ewp = east | west | position;

            let north = (ewp & !BB::RANK_8) << 8;
            let south = (ewp & !BB::RANK_1) >> 8;
            res[Square::new(i)] = north | south | east | west;
        }
        res
    }

    fn gen_ray_attackes() -> DirectionArray<BoardArray<BB>> {
        let mut res = DirectionArray::new(BoardArray::new(BB::empty()));
        for d in 0..8 {
            let mask = Self::DIRECTION_MASK[d];
            let shift = Self::DIRECTION_SHIFT[d];
            let d = Direction::from_u8(d as u8);
            for i in 0..64 {
                let i = Square::new(i);
                res[d][i] = BB::square(i);
                for _ in 0..7 {
                    let r = (res[d][i] & !mask).shift(shift);
                    res[d][i] |= r;
                }
                res[d][i] &= !BB::square(i)
            }
        }

        res
    }

    fn gen_between() -> BoardArray<BoardArray<BB>> {
        let mut res = BoardArray::new(BoardArray::new(BB::empty()));

        for i in 0..64 {
            for j in 0..64 {
                let file_first = i & 7;
                let rank_first = i >> 3;
                let file_sec = j & 7;
                let rank_sec = j >> 3;

                let i = Square::new(i);
                let j = Square::new(j);

                if file_first == file_sec && rank_first == rank_sec {
                    continue;
                }

                if file_first == file_sec {
                    let min = rank_first.min(rank_sec);
                    let max = rank_first.max(rank_sec);
                    for r in min..=max {
                        res[i][j] |= BB::square(Square::from_file_rank(file_first, r));
                    }
                }

                if rank_first == rank_sec {
                    let min = file_first.min(file_sec);
                    let max = file_first.max(file_sec);
                    for f in min..=max {
                        res[i][j] |= BB::square(Square::from_file_rank(f, rank_first));
                    }
                }

                if (rank_first as i8 - rank_sec as i8).abs()
                    == (file_first as i8 - file_sec as i8).abs()
                {
                    let len = (rank_first as i8 - rank_sec as i8).abs() as u8;
                    let rank_step = (rank_sec as i8 - rank_first as i8).signum();
                    let file_step = (file_sec as i8 - file_first as i8).signum();

                    for s in 0..=len as i8 {
                        let rank = rank_first as i8 + rank_step * s;
                        let file = file_first as i8 + file_step * s;
                        res[i][j] |= BB::square(Square::from_file_rank(file as u8, rank as u8));
                    }
                }

                res[i][j] &= !BB::square(i);
                res[i][j] &= !BB::square(j);
            }
        }

        res
    }

    pub fn gen_moves(&self, b: &Board, res: &mut Vec<Move>) {
        let player_pieces = b[Piece::WhiteKing]
            | b[Piece::WhiteQueen]
            | b[Piece::WhiteBishop]
            | b[Piece::WhiteKnight]
            | b[Piece::WhiteRook]
            | b[Piece::WhitePawn];

        let opponent_pieces = b[Piece::BlackKing]
            | b[Piece::BlackQueen]
            | b[Piece::BlackBishop]
            | b[Piece::BlackKnight]
            | b[Piece::BlackRook]
            | b[Piece::BlackPawn];

        let occupied = player_pieces | opponent_pieces;

        let attacked = self.attacked(b, occupied);
        let (rook_pinners, bishop_pinners, rook_pinned, bishop_pinned) =
            self.pinned(b, player_pieces, occupied);

        let pinned = bishop_pinned | rook_pinned;

        if (b[Piece::WhiteKing] & attacked).any() {
            self.gen_moves_check(b, attacked, player_pieces, opponent_pieces, occupied, res);
            return;
        }

        self.pawn_moves(b, occupied, pinned, opponent_pieces, res);
        self.sliding_pieces(b, occupied, pinned, player_pieces, res);

        for p in (b[Piece::WhiteKnight] & !pinned).iter() {
            let moves = self.knight_attacks[p] & !player_pieces;
            for m in moves.iter() {
                res.push(Move::Simple {
                    from: p,
                    to: m,
                    piece: Piece::WhiteKnight,
                })
            }
        }

        let p = b[Piece::WhiteKing].first_piece();
        let moves = self.king_attacks[p] & !player_pieces & !attacked;
        for m in moves.iter() {
            res.push(Move::Simple {
                from: p,
                to: m,
                piece: Piece::WhiteKing,
            })
        }

        if (b.state & ExtraState::WHITE_KING_CASTLE).any() {
            let present = occupied & BB::WHITE_KING_CASTLE_MASK;
            let attacked = attacked & BB(6);
            if (present | attacked).none() {
                res.push(Move::Castle { king: true })
            }
        }

        if (b.state & ExtraState::WHITE_QUEEN_CASTLE).any() {
            let present = occupied & BB::WHITE_QUEEN_CASTLE_MASK;
            let attacked = attacked & BB(2);
            if (present | attacked).none() {
                res.push(Move::Castle { king: false })
            }
        }

        self.move_pinned(
            b,
            occupied,
            rook_pinned,
            bishop_pinned,
            rook_pinners,
            bishop_pinners,
            res,
        );
    }

    fn gen_moves_check(
        &self,
        b: &Board,
        attacked: BB,
        player_pieces: BB,
        opponent_pieces: BB,
        occupied: BB,
        res: &mut Vec<Move>,
    ) {
        let king_square = b[Piece::WhiteKing].first_piece();

        let checkers = (self.knight_attacks[king_square]
            | self.rook_attacks(king_square, occupied)
            | self.bishop_attacks(king_square, occupied))
            & opponent_pieces;

        let p = b[Piece::WhiteKing].first_piece();
        let moves = self.king_attacks[p] & !player_pieces & !attacked;
        for m in moves.iter() {
            res.push(Move::Simple {
                from: p,
                to: m,
                piece: Piece::WhiteKing,
            })
        }

        if checkers.count() > 1 {
            return;
        }

        let checker = checkers.first_piece();

        // All moves which take the checker
        let checker_attacked_bishop = self.bishop_attacks(checker, occupied);
        for p in (checker_attacked_bishop & b[Piece::WhiteBishop]).iter() {
            res.push(Move::Simple {
                from: p,
                to: checker,
                piece: Piece::WhiteBishop,
            });
        }
        for p in (checker_attacked_bishop & b[Piece::WhiteQueen]).iter() {
            res.push(Move::Simple {
                from: p,
                to: checker,
                piece: Piece::WhiteQueen,
            });
        }

        let checker_attacked_rook = self.rook_attacks(checker, occupied);
        for p in (checker_attacked_rook & b[Piece::WhiteRook]).iter() {
            res.push(Move::Simple {
                from: p,
                to: checker,
                piece: Piece::WhiteRook,
            });
        }
        for p in (checker_attacked_rook & b[Piece::WhiteQueen]).iter() {
            res.push(Move::Simple {
                from: p,
                to: checker,
                piece: Piece::WhiteQueen,
            });
        }

        let checker_attacked_knight = self.knight_attacks[checker] & b[Piece::WhiteKnight];
        for p in checker_attacked_knight.iter() {
            res.push(Move::Simple {
                from: p,
                to: checker,
                piece: Piece::WhiteKnight,
            });
        }

        let pawn_attacks = (b[Piece::WhitePawn] << 7) & checkers;
        for p in pawn_attacks.iter() {
            res.push(Move::Simple {
                from: p - 7,
                to: p,
                piece: Piece::WhiteKnight,
            });
        }
        let pawn_attacks = (b[Piece::WhitePawn] << 9) & checkers;
        for p in pawn_attacks.iter() {
            res.push(Move::Simple {
                from: p - 9,
                to: p,
                piece: Piece::WhiteKnight,
            });
        }

        if (b[Piece::BlackKnight] & checkers).any() {
            return;
        }

        // All blocking moves
        let between = self.between[checker][king_square];

        let pawn_blocks = (b[Piece::WhitePawn] << 8) & between;
        for p in pawn_blocks.iter() {
            res.push(Move::Simple {
                from: p - 8,
                to: p,
                piece: Piece::WhitePawn,
            });
        }
        let double_pawn_blocks = ((b[Piece::WhitePawn] & BB::RANK_2) << 16) & between;
        for p in double_pawn_blocks.iter() {
            res.push(Move::Simple {
                from: p - 16,
                to: p,
                piece: Piece::WhitePawn,
            });
        }

        for block in between.iter() {
            let block_attacked_bishop = self.bishop_attacks(block, occupied);
            for p in (block_attacked_bishop & b[Piece::WhiteBishop]).iter() {
                res.push(Move::Simple {
                    from: p,
                    to: block,
                    piece: Piece::WhiteBishop,
                });
            }
            for p in (block_attacked_bishop & b[Piece::WhiteQueen]).iter() {
                res.push(Move::Simple {
                    from: p,
                    to: block,
                    piece: Piece::WhiteQueen,
                });
            }

            let block_attacked_rook = self.rook_attacks(block, occupied);
            for p in (block_attacked_rook & b[Piece::WhiteRook]).iter() {
                res.push(Move::Simple {
                    from: p,
                    to: block,
                    piece: Piece::WhiteRook,
                });
            }
            for p in (block_attacked_rook & b[Piece::WhiteQueen]).iter() {
                res.push(Move::Simple {
                    from: p,
                    to: block,
                    piece: Piece::WhiteQueen,
                });
            }

            let block_attacked_knight = self.knight_attacks[block] & b[Piece::WhiteKnight];
            for p in block_attacked_knight.iter() {
                res.push(Move::Simple {
                    from: p,
                    to: checker,
                    piece: Piece::WhiteKnight,
                });
            }
        }
    }

    fn move_pinned(
        &self,
        b: &Board,
        occupied: BB,
        rook_pinned: BB,
        bishop_pinned: BB,
        rook_pinners: BB,
        bishop_pinners: BB,
        res: &mut Vec<Move>,
    ) {
        for p in rook_pinned.iter() {
            let square = BB::square(p);
            if (b[Piece::WhiteQueen] & square).any() {
                let attack = self.ray_attacks_positive(p, occupied, Direction::N)
                    | self.ray_attacks_positive(p, occupied, Direction::S);
                let mut attacks = (attack & rook_pinners).saturate() & attack;
                let attack = self.ray_attacks_positive(p, occupied, Direction::W)
                    | self.ray_attacks_positive(p, occupied, Direction::E);
                attacks |= (attack & rook_pinners).saturate() & attack;

                for m in attacks.iter() {
                    res.push(Move::Simple {
                        from: p,
                        to: m,
                        piece: Piece::WhiteQueen,
                    })
                }
            }
            if (b[Piece::WhiteRook] & square).any() {
                let attack = self.ray_attacks_positive(p, occupied, Direction::N)
                    | self.ray_attacks_positive(p, occupied, Direction::S);
                let mut attacks = (attack & rook_pinners).saturate() & attack;
                let attack = self.ray_attacks_positive(p, occupied, Direction::W)
                    | self.ray_attacks_positive(p, occupied, Direction::E);
                attacks |= (attack & rook_pinners).saturate() & attack;

                for m in attacks.iter() {
                    res.push(Move::Simple {
                        from: p,
                        to: m,
                        piece: Piece::WhiteRook,
                    })
                }
            }
            if (b[Piece::WhitePawn] & square).any() {
                let move_mask = (BB::FILE_A << p.rank() & rook_pinners).saturate();
                let single_move = (square << 8) & !occupied;
                let double_move = ((single_move & BB::RANK_3) << 8) & !rook_pinners;
                for m in (move_mask & single_move & double_move).iter() {
                    res.push(Move::Simple {
                        from: p,
                        to: m,
                        piece: Piece::WhitePawn,
                    });
                }
            }
        }

        for p in bishop_pinned.iter() {
            let square = BB::square(p);
            if (b[Piece::WhiteQueen] & square).any() {
                let attack = self.ray_attacks_positive(p, occupied, Direction::NW)
                    | self.ray_attacks_positive(p, occupied, Direction::SE);
                let mut attacks = (attack & bishop_pinners).saturate() & attack;
                let attack = self.ray_attacks_positive(p, occupied, Direction::NE)
                    | self.ray_attacks_positive(p, occupied, Direction::SW);
                attacks |= (attack & bishop_pinners).saturate() & attack;

                for m in attacks.iter() {
                    res.push(Move::Simple {
                        from: p,
                        to: m,
                        piece: Piece::WhiteQueen,
                    })
                }
            }
            if (b[Piece::WhiteBishop] & square).any() {
                let attack = self.ray_attacks_positive(p, occupied, Direction::NW)
                    | self.ray_attacks_positive(p, occupied, Direction::SE);
                let mut attacks = (attack & bishop_pinners).saturate() & attack;
                let attack = self.ray_attacks_positive(p, occupied, Direction::NE)
                    | self.ray_attacks_positive(p, occupied, Direction::SW);
                attacks |= (attack & bishop_pinners).saturate() & attack;

                for m in attacks.iter() {
                    res.push(Move::Simple {
                        from: p,
                        to: m,
                        piece: Piece::WhiteBishop,
                    })
                }
            }

            if (b[Piece::WhitePawn] & square).any() {
                let moves = ((square << 7) & bishop_pinners) | (square << 9 & bishop_pinners);
                if moves.any() {
                    res.push(Move::Simple {
                        from: p,
                        to: moves.first_piece(),
                        piece: Piece::WhitePawn,
                    })
                }
            }
        }
    }

    fn attacked(&self, b: &Board, occupied: BB) -> BB {
        let mut attacked = BB::empty();

        for p in b[Piece::BlackKing].iter() {
            attacked |= self.king_attacks[p]
        }
        for p in b[Piece::BlackKnight].iter() {
            attacked |= self.knight_attacks[p]
        }

        let empty = !occupied;
        let diagonal = b[Piece::BlackQueen] | b[Piece::BlackBishop];
        attacked |= fill_7::nw(diagonal, empty);
        attacked |= fill_7::ne(diagonal, empty);
        attacked |= fill_7::sw(diagonal, empty);
        attacked |= fill_7::se(diagonal, empty);

        let straight = b[Piece::BlackQueen] | b[Piece::BlackRook];
        attacked |= fill_7::n(straight, empty);
        attacked |= fill_7::w(straight, empty);
        attacked |= fill_7::s(straight, empty);
        attacked |= fill_7::e(straight, empty);

        let pawn_attacks =
            (b[Piece::BlackPawn] & !BB::FILE_H) >> 7 | (b[Piece::BlackPawn] & !BB::FILE_A) >> 9;
        attacked | pawn_attacks
    }

    fn pinned(&self, b: &Board, player: BB, occupied: BB) -> (BB, BB, BB, BB) {
        let king_square = b[Piece::WhiteKing].first_piece();
        let rook_pinners = self.xray_rook_attacks(king_square, player, occupied)
            & (b[Piece::BlackRook] | b[Piece::BlackQueen]);

        let mut rook_pinned = BB::empty();
        for p in rook_pinners.iter() {
            let between = self.between[king_square][p];
            rook_pinned |= player & between;
        }

        let bishop_pinners = self.xray_bishop_attacks(king_square, player, occupied)
            & (b[Piece::BlackBishop] | b[Piece::BlackQueen]);

        let mut bishop_pinned = BB::empty();
        for p in bishop_pinners.iter() {
            let between = self.between[king_square][p];
            bishop_pinned |= player & between;
        }

        (rook_pinners, bishop_pinners, rook_pinned, bishop_pinned)
    }

    fn pawn_moves(
        &self,
        b: &Board,
        occupied: BB,
        pinned: BB,
        opponent_pieces: BB,
        buffer: &mut Vec<Move>,
    ) {
        let pawns = b[Piece::WhitePawn];
        let free_pawns = pawns & !pinned;

        let left_pawn_attacks = ((free_pawns & !BB::FILE_A) << 9) & opponent_pieces;
        let left_pawn_attacks_promote = left_pawn_attacks & BB::RANK_8;
        let left_pawn_attacks = left_pawn_attacks & !BB::RANK_8;
        for p in left_pawn_attacks.iter() {
            buffer.push(Move::Simple {
                from: p - 9,
                to: p,
                piece: Piece::WhitePawn,
            });
        }
        for p in left_pawn_attacks_promote.iter() {
            Self::add_all_promotions(p, p - 9, buffer);
        }

        let right_pawn_attacks = ((free_pawns & !BB::FILE_H) << 7) & opponent_pieces;
        let right_pawn_attacks_promote = right_pawn_attacks & BB::RANK_8;
        let right_pawn_attacks = right_pawn_attacks & !BB::RANK_8;
        for p in right_pawn_attacks.iter() {
            buffer.push(Move::Simple {
                from: p - 7,
                to: p,
                piece: Piece::WhitePawn,
            });
        }
        for p in right_pawn_attacks_promote.iter() {
            Self::add_all_promotions(p, p - 7, buffer);
        }

        let pawn_moves = ((free_pawns & !pinned) << 8) & !occupied;
        let pawn_moves_promote = right_pawn_attacks & BB::RANK_8;
        let pawn_moves = pawn_moves & !BB::RANK_8;
        for p in pawn_moves.iter() {
            buffer.push(Move::Simple {
                from: p - 8,
                to: p,
                piece: Piece::WhitePawn,
            });
        }
        for p in pawn_moves_promote.iter() {
            Self::add_all_promotions(p, p - 8, buffer);
        }

        let double_pawn_moves = ((pawn_moves & BB::RANK_3) << 8) & !occupied;
        for p in double_pawn_moves.iter() {
            buffer.push(Move::Simple {
                from: p - 16,
                to: p,
                piece: Piece::WhitePawn,
            });
        }

        if let Some(_) = b.state.get_en_passant() {
            todo!()
        }
    }

    fn add_all_promotions(to: Square, from: Square, buffer: &mut Vec<Move>) {
        buffer.push(Move::Promote {
            promote: Piece::WhiteBishop,
            from,
            to,
        });
        buffer.push(Move::Promote {
            promote: Piece::WhiteRook,
            from,
            to,
        });
        buffer.push(Move::Promote {
            promote: Piece::WhiteKnight,
            from,
            to,
        });
        buffer.push(Move::Promote {
            promote: Piece::WhiteQueen,
            from,
            to,
        });
    }

    fn sliding_pieces(
        &self,
        b: &Board,
        occupied: BB,
        pinned: BB,
        player: BB,
        buffer: &mut Vec<Move>,
    ) {
        for p in (b[Piece::WhiteBishop] & !pinned).iter() {
            let mut attacks = self.bishop_attacks(p, occupied);
            attacks &= !player;

            for m in attacks.iter() {
                buffer.push(Move::Simple {
                    from: p,
                    to: m,
                    piece: Piece::WhiteBishop,
                })
            }
        }

        for p in (b[Piece::WhiteRook] & !pinned).iter() {
            let attacks = self.rook_attacks(p, occupied) & !player;

            for m in attacks.iter() {
                buffer.push(Move::Simple {
                    from: p,
                    to: m,
                    piece: Piece::WhiteRook,
                })
            }
        }

        for p in (b[Piece::WhiteQueen] & !pinned).iter() {
            let attacks = (self.rook_attacks(p, occupied) & !player
                | self.bishop_attacks(p, occupied))
                & !player;

            for m in attacks.iter() {
                buffer.push(Move::Simple {
                    from: p,
                    to: m,
                    piece: Piece::WhiteQueen,
                })
            }
        }
    }

    fn xray_rook_attacks(&self, square: Square, mut blockers: BB, occupied: BB) -> BB {
        let attacks = self.rook_attacks(square, occupied);
        blockers &= attacks;
        attacks ^ self.rook_attacks(square, occupied ^ blockers)
    }

    fn xray_bishop_attacks(&self, square: Square, mut blockers: BB, occupied: BB) -> BB {
        let attacks = self.bishop_attacks(square, occupied);
        blockers &= attacks;
        attacks ^ self.bishop_attacks(square, occupied ^ blockers)
    }

    fn rook_attacks(&self, square: Square, occupied: BB) -> BB {
        self.ray_attacks_positive(square, occupied, Direction::N)
            | self.ray_attacks_positive(square, occupied, Direction::E)
            | self.ray_attacks_negative(square, occupied, Direction::S)
            | self.ray_attacks_negative(square, occupied, Direction::W)
    }

    fn bishop_attacks(&self, square: Square, occupied: BB) -> BB {
        self.ray_attacks_positive(square, occupied, Direction::NW)
            | self.ray_attacks_positive(square, occupied, Direction::NE)
            | self.ray_attacks_negative(square, occupied, Direction::SE)
            | self.ray_attacks_negative(square, occupied, Direction::SW)
    }

    fn ray_attacks_positive(&self, square: Square, occupied: BB, direction: Direction) -> BB {
        let attack = self.ray_attacks[direction][square];
        let blockers = attack & occupied;
        let block_square = (blockers | BB::B8).first_piece();
        attack ^ self.ray_attacks[direction][block_square]
    }

    fn ray_attacks_negative(&self, square: Square, occupied: BB, direction: Direction) -> BB {
        let attack = self.ray_attacks[direction][square];
        let blockers = attack & occupied;
        let block_square = (blockers | BB::A1).last_piece();
        attack ^ self.ray_attacks[direction][block_square]
    }
}
