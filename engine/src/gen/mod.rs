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

pub trait Player {
    type Opponent: Player;

    const KING: Piece;
    const QUEEN: Piece;
    const BISHOP: Piece;
    const KNIGHT: Piece;
    const ROOK: Piece;
    const PAWN: Piece;

    const KING_CASTLE_STATE: ExtraState;
    const KING_CASTLE_TARGET: BB;
    const QUEEN_CASTLE_STATE: ExtraState;
    const QUEEN_CASTLE_TARGET: BB;

    const KING_CASTLE_MASK: BB;
    const QUEEN_CASTLE_MASK: BB;

    const PROMOTE_RANK: BB;
    const DOUBLE_MOVE_RANK: BB;
    const LEFT_FILE: BB;
    const RIGHT_FILE: BB;

    const PAWN_MOVE: i8;
    const PAWN_ATTACK_LEFT: i8;
    const PAWN_ATTACK_RIGHT: i8;

    fn pawn_move(b: BB) -> BB;

    fn pawn_attacks_left(b: BB) -> BB;

    fn pawn_attacks_right(b: BB) -> BB;
}

struct White;

impl Player for White {
    type Opponent = Black;

    const KING: Piece = Piece::WhiteKing;
    const QUEEN: Piece = Piece::WhiteQueen;
    const BISHOP: Piece = Piece::WhiteBishop;
    const KNIGHT: Piece = Piece::WhiteKnight;
    const ROOK: Piece = Piece::WhiteRook;
    const PAWN: Piece = Piece::WhitePawn;

    const KING_CASTLE_STATE: ExtraState = ExtraState::WHITE_KING_CASTLE;
    const KING_CASTLE_TARGET: BB = BB(6);
    const QUEEN_CASTLE_STATE: ExtraState = ExtraState::WHITE_QUEEN_CASTLE;
    const QUEEN_CASTLE_TARGET: BB = BB(2);

    const KING_CASTLE_MASK: BB = BB(0b1100000);
    const QUEEN_CASTLE_MASK: BB = BB(0b1110);

    const PROMOTE_RANK: BB = BB::RANK_8;
    const DOUBLE_MOVE_RANK: BB = BB::RANK_3;
    const LEFT_FILE: BB = BB::FILE_A;
    const RIGHT_FILE: BB = BB::FILE_H;

    const PAWN_MOVE: i8 = 8;
    const PAWN_ATTACK_LEFT: i8 = 7;
    const PAWN_ATTACK_RIGHT: i8 = 9;

    #[inline]
    fn pawn_move(b: BB) -> BB {
        b << 8
    }

    #[inline]
    fn pawn_attacks_left(b: BB) -> BB {
        (b & !BB::FILE_A) << 7
    }

    #[inline]
    fn pawn_attacks_right(b: BB) -> BB {
        (b & !BB::FILE_H) << 9
    }
}

struct Black;

impl Player for Black {
    type Opponent = White;

    const KING: Piece = Piece::BlackKing;
    const QUEEN: Piece = Piece::BlackQueen;
    const BISHOP: Piece = Piece::BlackBishop;
    const KNIGHT: Piece = Piece::BlackKnight;
    const ROOK: Piece = Piece::BlackRook;
    const PAWN: Piece = Piece::BlackPawn;

    const KING_CASTLE_STATE: ExtraState = ExtraState::BLACK_KING_CASTLE;
    const QUEEN_CASTLE_STATE: ExtraState = ExtraState::BLACK_QUEEN_CASTLE;

    const KING_CASTLE_MASK: BB = BB(0b1100000 << 56);
    const KING_CASTLE_TARGET: BB = BB(6 + 56);
    const QUEEN_CASTLE_MASK: BB = BB(0b1110 << 56);
    const QUEEN_CASTLE_TARGET: BB = BB(2 + 56);

    const PROMOTE_RANK: BB = BB::RANK_1;
    const DOUBLE_MOVE_RANK: BB = BB::RANK_6;
    const LEFT_FILE: BB = BB::FILE_H;
    const RIGHT_FILE: BB = BB::FILE_A;

    const PAWN_MOVE: i8 = -8;
    const PAWN_ATTACK_LEFT: i8 = -7;
    const PAWN_ATTACK_RIGHT: i8 = -9;

    #[inline]
    fn pawn_move(b: BB) -> BB {
        b >> 8
    }

    #[inline]
    fn pawn_attacks_left(b: BB) -> BB {
        (b & !BB::FILE_H) >> 7
    }

    #[inline]
    fn pawn_attacks_right(b: BB) -> BB {
        (b & !BB::FILE_A) >> 9
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
        if b.white_turn() {
            self.gen_moves_player::<White>(b, res);
        } else {
            self.gen_moves_player::<Black>(b, res);
        }
    }

    pub fn gen_moves_player<P: Player>(&self, b: &Board, res: &mut Vec<Move>) {
        let player_pieces =
            b[P::KING] | b[P::QUEEN] | b[P::BISHOP] | b[P::KNIGHT] | b[P::ROOK] | b[P::PAWN];

        let opponent_pieces = b[P::Opponent::KING]
            | b[P::Opponent::QUEEN]
            | b[P::Opponent::BISHOP]
            | b[P::Opponent::KNIGHT]
            | b[P::Opponent::ROOK]
            | b[P::Opponent::PAWN];

        let occupied = player_pieces | opponent_pieces;

        let attacked = self.attacked::<P>(b, occupied);
        let (rook_pinners, bishop_pinners, rook_pinned, bishop_pinned) =
            self.pinned::<P>(b, player_pieces, occupied);

        let pinned = bishop_pinned | rook_pinned;

        if (b[P::KING] & attacked).any() {
            self.gen_moves_check::<P>(b, player_pieces, opponent_pieces, occupied, pinned, res);
            return;
        }

        self.pawn_moves::<P>(b, occupied, pinned, opponent_pieces, res);
        self.sliding_pieces::<P>(b, occupied, pinned, player_pieces, res);

        for p in (b[P::KNIGHT] & !pinned).iter() {
            let moves = self.knight_attacks[p] & !player_pieces;
            for m in moves.iter() {
                res.push(Move::Simple {
                    from: p,
                    to: m,
                    piece: P::KNIGHT,
                })
            }
        }

        let p = b[P::KING].first_piece();
        let moves = self.king_attacks[p] & !player_pieces & !attacked;
        for m in moves.iter() {
            res.push(Move::Simple {
                from: p,
                to: m,
                piece: P::KING,
            })
        }

        if (b.state & P::KING_CASTLE_STATE).any() {
            let present = occupied & P::KING_CASTLE_MASK;
            let attacked = attacked & P::KING_CASTLE_TARGET;
            if (present | attacked).none() {
                res.push(Move::Castle { king: true })
            }
        }

        if (b.state & P::QUEEN_CASTLE_STATE).any() {
            let present = occupied & P::QUEEN_CASTLE_MASK;
            let attacked = attacked & P::QUEEN_CASTLE_TARGET;
            if (present | attacked).none() {
                res.push(Move::Castle { king: false })
            }
        }

        self.move_pinned::<P>(
            b,
            occupied,
            player_pieces,
            rook_pinned,
            bishop_pinned,
            rook_pinners,
            bishop_pinners,
            res,
        );
    }

    fn gen_moves_check<P: Player>(
        &self,
        b: &Board,
        player_pieces: BB,
        opponent_pieces: BB,
        occupied: BB,
        pinned: BB,
        res: &mut Vec<Move>,
    ) {
        let king_square = b[P::KING].first_piece();

        let attacked = self.attacked::<P>(b, occupied & !b[P::KING]);

        let checkers = (self.knight_attacks[king_square] & b[P::Opponent::KNIGHT]
            | self.rook_attacks(king_square, occupied)
                & (b[P::Opponent::ROOK] | b[P::Opponent::QUEEN])
            | self.bishop_attacks(king_square, occupied)
                & (b[P::Opponent::BISHOP] | b[P::Opponent::QUEEN]))
            | (P::pawn_attacks_left(b[P::KING]) | P::pawn_attacks_right(b[P::KING]))
                & b[P::Opponent::PAWN];
        &opponent_pieces;

        let p = b[P::KING].first_piece();
        let moves = self.king_attacks[p] & !player_pieces & !attacked;
        for m in moves.iter() {
            res.push(Move::Simple {
                from: p,
                to: m,
                piece: P::KING,
            })
        }

        if checkers.count() > 1 {
            return;
        }

        let checker = checkers.first_piece();

        // All moves which take the checker
        let checker_attacked_bishop = self.bishop_attacks(checker, occupied);
        for p in (checker_attacked_bishop & b[P::BISHOP] & !pinned).iter() {
            res.push(Move::Simple {
                from: p,
                to: checker,
                piece: P::BISHOP,
            });
        }
        for p in (checker_attacked_bishop & b[P::QUEEN] & !pinned).iter() {
            res.push(Move::Simple {
                from: p,
                to: checker,
                piece: P::QUEEN,
            });
        }

        let checker_attacked_rook = self.rook_attacks(checker, occupied);
        for p in (checker_attacked_rook & b[P::ROOK] & !pinned).iter() {
            res.push(Move::Simple {
                from: p,
                to: checker,
                piece: P::ROOK,
            });
        }
        for p in (checker_attacked_rook & b[P::QUEEN] & !pinned).iter() {
            res.push(Move::Simple {
                from: p,
                to: checker,
                piece: P::QUEEN,
            });
        }

        let checker_attacked_knight = self.knight_attacks[checker] & b[P::KNIGHT] & !pinned;
        for p in checker_attacked_knight.iter() {
            res.push(Move::Simple {
                from: p,
                to: checker,
                piece: P::KNIGHT,
            });
        }

        let pawn_attacks = P::pawn_attacks_left(b[P::PAWN] & !pinned) & checkers;
        for p in pawn_attacks.iter() {
            res.push(Move::Simple {
                from: p - P::PAWN_ATTACK_LEFT,
                to: p,
                piece: P::PAWN,
            });
        }
        let pawn_attacks = P::pawn_attacks_right(b[P::PAWN] & !pinned) & checkers;
        for p in pawn_attacks.iter() {
            res.push(Move::Simple {
                from: p - P::PAWN_ATTACK_RIGHT,
                to: p,
                piece: P::PAWN,
            });
        }

        if (b[P::Opponent::KNIGHT] & checkers).any() {
            return;
        }

        // All blocking moves
        let between = self.between[checker][king_square];

        let pawn_blocks = P::pawn_move(b[P::PAWN] & !pinned) & between;
        for p in pawn_blocks.iter() {
            res.push(Move::Simple {
                from: p - P::PAWN_MOVE,
                to: p,
                piece: P::PAWN,
            });
        }
        let double_pawn_blocks =
            P::pawn_move(P::pawn_move(b[P::PAWN] & !pinned) & P::DOUBLE_MOVE_RANK & !occupied)
                & between;
        for p in double_pawn_blocks.iter() {
            res.push(Move::Simple {
                from: p - (P::PAWN_MOVE + P::PAWN_MOVE),
                to: p,
                piece: P::PAWN,
            });
        }

        for block in between.iter() {
            let block_attacked_bishop = self.bishop_attacks(block, occupied);
            for p in (block_attacked_bishop & b[P::BISHOP] & !pinned).iter() {
                res.push(Move::Simple {
                    from: p,
                    to: block,
                    piece: P::BISHOP,
                });
            }
            for p in (block_attacked_bishop & b[P::QUEEN] & !pinned).iter() {
                res.push(Move::Simple {
                    from: p,
                    to: block,
                    piece: P::QUEEN,
                });
            }

            let block_attacked_rook = self.rook_attacks(block, occupied);
            for p in (block_attacked_rook & b[P::ROOK] & !pinned).iter() {
                res.push(Move::Simple {
                    from: p,
                    to: block,
                    piece: P::ROOK,
                });
            }
            for p in (block_attacked_rook & b[P::QUEEN] & !pinned).iter() {
                res.push(Move::Simple {
                    from: p,
                    to: block,
                    piece: P::QUEEN,
                });
            }

            let block_attacked_knight = self.knight_attacks[block] & b[P::KNIGHT] & !pinned;
            for p in block_attacked_knight.iter() {
                res.push(Move::Simple {
                    from: p,
                    to: block,
                    piece: P::KNIGHT,
                });
            }
        }
    }

    fn move_pinned<P: Player>(
        &self,
        b: &Board,
        occupied: BB,
        player_pieces: BB,
        rook_pinned: BB,
        bishop_pinned: BB,
        rook_pinners: BB,
        bishop_pinners: BB,
        res: &mut Vec<Move>,
    ) {
        for p in rook_pinned.iter() {
            let square = BB::square(p);
            if (b[P::QUEEN] & square).any() {
                let attack = self.ray_attacks_positive(p, occupied, Direction::N)
                    | self.ray_attacks_negative(p, occupied, Direction::S);
                let mut attacks = (attack & rook_pinners).saturate() & attack;
                let attack = self.ray_attacks_negative(p, occupied, Direction::W)
                    | self.ray_attacks_positive(p, occupied, Direction::E);
                attacks |= (attack & rook_pinners).saturate() & attack;
                attacks &= !player_pieces;

                for m in attacks.iter() {
                    res.push(Move::Simple {
                        from: p,
                        to: m,
                        piece: P::QUEEN,
                    })
                }
            }
            if (b[P::ROOK] & square).any() {
                let attack = self.ray_attacks_positive(p, occupied, Direction::N)
                    | self.ray_attacks_negative(p, occupied, Direction::S);
                let mut attacks = (attack & rook_pinners).saturate() & attack;
                let attack = self.ray_attacks_negative(p, occupied, Direction::W)
                    | self.ray_attacks_positive(p, occupied, Direction::E);
                attacks |= (attack & rook_pinners).saturate() & attack;
                attacks &= !player_pieces;

                for m in attacks.iter() {
                    res.push(Move::Simple {
                        from: p,
                        to: m,
                        piece: P::ROOK,
                    })
                }
            }
            if (b[P::PAWN] & square).any() {
                let move_mask = (BB::FILE_A << p.file() & rook_pinners).saturate();
                let single_move = P::pawn_move(square) & !occupied;
                let double_move = P::pawn_move(single_move & P::DOUBLE_MOVE_RANK) & !occupied;
                for m in (move_mask & (single_move | double_move)).iter() {
                    res.push(Move::Simple {
                        from: p,
                        to: m,
                        piece: P::PAWN,
                    });
                }
            }
        }

        for p in bishop_pinned.iter() {
            let square = BB::square(p);
            if (b[P::QUEEN] & square).any() {
                let attack = self.ray_attacks_positive(p, occupied, Direction::NW)
                    | self.ray_attacks_negative(p, occupied, Direction::SE);
                let mut attacks = (attack & bishop_pinners).saturate() & attack;
                let attack = self.ray_attacks_positive(p, occupied, Direction::NE)
                    | self.ray_attacks_negative(p, occupied, Direction::SW);
                attacks |= (attack & bishop_pinners).saturate() & attack;
                attacks &= !player_pieces;

                for m in attacks.iter() {
                    res.push(Move::Simple {
                        from: p,
                        to: m,
                        piece: P::QUEEN,
                    })
                }
            }
            if (b[P::BISHOP] & square).any() {
                let attack = self.ray_attacks_positive(p, occupied, Direction::NW)
                    | self.ray_attacks_negative(p, occupied, Direction::SE);
                let mut attacks = (attack & bishop_pinners).saturate() & attack;
                let attack = self.ray_attacks_positive(p, occupied, Direction::NE)
                    | self.ray_attacks_negative(p, occupied, Direction::SW);
                attacks |= (attack & bishop_pinners).saturate() & attack;
                attacks &= !player_pieces;

                for m in attacks.iter() {
                    res.push(Move::Simple {
                        from: p,
                        to: m,
                        piece: P::BISHOP,
                    })
                }
            }

            if (b[P::PAWN] & square).any() {
                let moves = P::pawn_attacks_left(square) & bishop_pinners
                    | P::pawn_attacks_right(square) & bishop_pinners;
                if moves.any() {
                    res.push(Move::Simple {
                        from: p,
                        to: moves.first_piece(),
                        piece: P::PAWN,
                    })
                }
            }
        }
    }

    fn attacked<P: Player>(&self, b: &Board, occupied: BB) -> BB {
        let mut attacked = BB::empty();

        for p in b[P::Opponent::KING].iter() {
            attacked |= self.king_attacks[p]
        }
        for p in b[P::Opponent::KNIGHT].iter() {
            attacked |= self.knight_attacks[p]
        }

        let empty = !occupied;
        let diagonal = b[P::Opponent::QUEEN] | b[P::Opponent::BISHOP];
        attacked |= fill_7::nw(diagonal, empty);
        attacked |= fill_7::ne(diagonal, empty);
        attacked |= fill_7::sw(diagonal, empty);
        attacked |= fill_7::se(diagonal, empty);

        let straight = b[P::Opponent::QUEEN] | b[P::Opponent::ROOK];
        attacked |= fill_7::n(straight, empty);
        attacked |= fill_7::w(straight, empty);
        attacked |= fill_7::s(straight, empty);
        attacked |= fill_7::e(straight, empty);

        let pawn_attacks = P::Opponent::pawn_attacks_left(b[P::Opponent::PAWN])
            | P::Opponent::pawn_attacks_right(b[P::Opponent::PAWN]);
        attacked | pawn_attacks
    }

    fn pinned<P: Player>(&self, b: &Board, player: BB, occupied: BB) -> (BB, BB, BB, BB) {
        let king_square = b[P::KING].first_piece();
        let rook_pinners = self.xray_rook_attacks(king_square, player, occupied)
            & (b[P::Opponent::ROOK] | b[P::Opponent::QUEEN]);

        let mut rook_pinned = BB::empty();
        for p in rook_pinners.iter() {
            let between = self.between[king_square][p];
            rook_pinned |= player & between;
        }

        let bishop_pinners = self.xray_bishop_attacks(king_square, player, occupied)
            & (b[P::Opponent::BISHOP] | b[P::Opponent::QUEEN]);

        let mut bishop_pinned = BB::empty();
        for p in bishop_pinners.iter() {
            let between = self.between[king_square][p];
            bishop_pinned |= player & between;
        }

        (rook_pinners, bishop_pinners, rook_pinned, bishop_pinned)
    }

    fn pawn_moves<P: Player>(
        &self,
        b: &Board,
        occupied: BB,
        pinned: BB,
        opponent_pieces: BB,
        buffer: &mut Vec<Move>,
    ) {
        let pawns = b[P::PAWN];
        let free_pawns = pawns & !pinned;

        let left_pawn_attacks = P::pawn_attacks_left(free_pawns) & opponent_pieces;
        let left_pawn_attacks_promote = left_pawn_attacks & P::PROMOTE_RANK;
        let left_pawn_attacks = left_pawn_attacks & !P::PROMOTE_RANK;
        for p in left_pawn_attacks.iter() {
            buffer.push(Move::Simple {
                from: p - P::PAWN_ATTACK_LEFT,
                to: p,
                piece: P::PAWN,
            });
        }
        for p in left_pawn_attacks_promote.iter() {
            Self::add_all_promotions::<P>(p, p - P::PAWN_ATTACK_LEFT, buffer);
        }

        let right_pawn_attacks = P::pawn_attacks_right(free_pawns) & opponent_pieces;
        let right_pawn_attacks_promote = right_pawn_attacks & P::PROMOTE_RANK;
        let right_pawn_attacks = right_pawn_attacks & !P::PROMOTE_RANK;
        for p in right_pawn_attacks.iter() {
            buffer.push(Move::Simple {
                from: p - P::PAWN_ATTACK_RIGHT,
                to: p,
                piece: P::PAWN,
            });
        }
        for p in right_pawn_attacks_promote.iter() {
            Self::add_all_promotions::<P>(p, p - P::PAWN_ATTACK_RIGHT, buffer);
        }

        let pawn_moves = P::pawn_move(free_pawns & !pinned) & !occupied;
        let pawn_moves_promote = pawn_moves & P::PROMOTE_RANK;
        let pawn_moves = pawn_moves & !P::PROMOTE_RANK;
        for p in pawn_moves.iter() {
            buffer.push(Move::Simple {
                from: p - P::PAWN_MOVE,
                to: p,
                piece: P::PAWN,
            });
        }
        for p in pawn_moves_promote.iter() {
            Self::add_all_promotions::<P>(p, p - P::PAWN_MOVE, buffer);
        }

        let double_pawn_moves = P::pawn_move(pawn_moves & P::DOUBLE_MOVE_RANK) & !occupied;
        for p in double_pawn_moves.iter() {
            buffer.push(Move::Simple {
                from: p - (P::PAWN_MOVE + P::PAWN_MOVE),
                to: p,
                piece: P::PAWN,
            });
        }

        if let Some(_) = b.state.get_en_passant() {
            todo!()
        }
    }

    fn add_all_promotions<P: Player>(to: Square, from: Square, buffer: &mut Vec<Move>) {
        buffer.push(Move::Promote {
            promote: P::BISHOP,
            from,
            to,
        });
        buffer.push(Move::Promote {
            promote: P::ROOK,
            from,
            to,
        });
        buffer.push(Move::Promote {
            promote: P::KNIGHT,
            from,
            to,
        });
        buffer.push(Move::Promote {
            promote: P::QUEEN,
            from,
            to,
        });
    }

    fn sliding_pieces<P: Player>(
        &self,
        b: &Board,
        occupied: BB,
        pinned: BB,
        player: BB,
        buffer: &mut Vec<Move>,
    ) {
        for p in (b[P::BISHOP] & !pinned).iter() {
            let mut attacks = self.bishop_attacks(p, occupied);
            attacks &= !player;

            for m in attacks.iter() {
                buffer.push(Move::Simple {
                    from: p,
                    to: m,
                    piece: P::BISHOP,
                })
            }
        }

        for p in (b[P::ROOK] & !pinned).iter() {
            let attacks = self.rook_attacks(p, occupied) & !player;

            for m in attacks.iter() {
                buffer.push(Move::Simple {
                    from: p,
                    to: m,
                    piece: P::ROOK,
                })
            }
        }

        for p in (b[P::QUEEN] & !pinned).iter() {
            let attacks = (self.rook_attacks(p, occupied) & !player
                | self.bishop_attacks(p, occupied))
                & !player;

            for m in attacks.iter() {
                buffer.push(Move::Simple {
                    from: p,
                    to: m,
                    piece: P::QUEEN,
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
