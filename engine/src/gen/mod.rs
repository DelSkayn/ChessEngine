use super::*;
mod util;

use util::{BoardArray, DirectionArray};
mod fill_7;

pub struct MoveGenerator {
    knight_attacks: BoardArray<BB>,
    king_attacks: BoardArray<BB>,
    ray_attacks: DirectionArray<BoardArray<BB>>,
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
        let pinned = self.pinned(b, player_pieces, occupied);
        dbg!(attacked);
        dbg!(pinned);

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

    fn pinned(&self, b: &Board, player: BB, occupied: BB) -> BB {
        let king_square = b[Piece::WhiteKing].first_piece();
        let mut pinners = self.xray_rook_attacks(king_square, player, occupied)
            & (b[Piece::BlackRook] | b[Piece::BlackQueen]);
        pinners |= self.xray_bishop_attacks(king_square, player, occupied)
            & (b[Piece::BlackBishop] | b[Piece::BlackQueen]);
        pinners
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
            let attacks =
                self.rook_attacks(p, occupied) | self.bishop_attacks(p, occupied) & !player;

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
