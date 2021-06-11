//! Move generation implementation

use super::util::{BoardArray, DirectionArray};
use super::{Board, Direction, ExtraState, Move, Piece, Square, BB};
use std::mem::MaybeUninit;

pub mod fill_7;

pub trait MoveBuffer {
    fn push(&mut self, mov: Move);

    fn swap(&mut self, first: usize, second: usize);

    fn len(&self) -> usize;

    fn get(&self, which: usize) -> &Move;
}

pub struct InlineBuffer {
    v: [MaybeUninit<Move>; 256],
    len: usize,
}

impl MoveBuffer for InlineBuffer {
    #[inline(always)]
    fn push(&mut self, mov: Move) {
        self.v[self.len] = MaybeUninit::new(mov);
        self.len += 1;
    }

    #[inline(always)]
    fn swap(&mut self, first: usize, second: usize) {
        self.v.swap(first, second);
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    fn get(&self, which: usize) -> &Move {
        assert!(self.len > which);
        unsafe { &*self.v.get_unchecked(which).as_ptr() }
    }
}

impl InlineBuffer {
    pub fn new() -> InlineBuffer {
        InlineBuffer {
            v: [MaybeUninit::uninit(); 256],
            len: 0,
        }
    }

    pub fn iter(&self) -> InlineIter {
        InlineIter {
            len: self.len,
            cur: 0,
            v: &self.v,
        }
    }
}

impl MoveBuffer for Vec<Move> {
    #[inline(always)]
    fn push(&mut self, mov: Move) {
        (*self).push(mov)
    }

    #[inline(always)]
    fn len(&self) -> usize {
        (*self).len()
    }

    #[inline(always)]
    fn swap(&mut self, first: usize, second: usize) {
        (**self).swap(first, second)
    }

    #[inline(always)]
    fn get(&self, which: usize) -> &Move {
        &self[which]
    }
}

impl<'a> Iterator for InlineIter<'a> {
    type Item = &'a Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == self.cur {
            return None;
        }
        let res = unsafe { &*self.v.get_unchecked(self.cur).as_ptr() };
        self.cur += 1;
        Some(res)
    }
}

pub struct MoveGenerator {
    knight_attacks: BoardArray<BB>,
    king_attacks: BoardArray<BB>,
    ray_attacks: DirectionArray<BoardArray<BB>>,
    between: BoardArray<BoardArray<BB>>,
}

impl MoveGenerator {
    const CASTLE_KING_ATTACKED_MASK: BB = BB(0b01100000);
    const CASTLE_QUEEN_ATTACKED_MASK: BB = BB(0b00001100);
    const CASTLE_KING_EMPTY_MASK: BB = BB(0b01100000);
    const CASTLE_QUEEN_EMPTY_MASK: BB = BB(0b00001110);

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
            for i in 0..64 {
                let d = Direction::from_u8(d);
                let i = Square::new(i);
                res[d][i] = BB::square(i);
                for _ in 0..7 {
                    let r = res[d][i].shift(d);
                    res[d][i] |= r;
                }
                res[d][i] &= !BB::square(i);
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
        let block_square = (blockers | BB::H8).first_piece();
        attack ^ self.ray_attacks[direction][block_square]
    }

    fn ray_attacks_negative(&self, square: Square, occupied: BB, direction: Direction) -> BB {
        let attack = self.ray_attacks[direction][square];
        let blockers = attack & occupied;
        let block_square = (blockers | BB::A1).last_piece();
        attack ^ self.ray_attacks[direction][block_square]
    }

    pub fn check_mate(&self, b: &Board) -> bool {
        let black_turn = b.state.black_turn;
        let white_turn = !black_turn;
        let king = Piece::player_king(black_turn);
        let knight = Piece::player_knight(black_turn);
        let queen = Piece::player_queen(black_turn);
        let rook = Piece::player_rook(black_turn);
        let bishop = Piece::player_bishop(black_turn);
        let pawn = Piece::player_pawn(black_turn);

        let king_square = b[king].first_piece();
        let their_queen = Piece::player_queen(white_turn);
        let their_pawn = Piece::player_pawn(white_turn);
        let their_bishop = Piece::player_bishop(white_turn);
        let their_rook = Piece::player_rook(white_turn);
        let their_knight = Piece::player_knight(white_turn);
        let their_rook_like = b[their_queen] | b[their_rook];
        let their_bishop_like = b[their_queen] | b[their_bishop];

        let mut occupied = BB::empty();
        let mut my = BB::empty();
        let mut their = BB::empty();

        for p in Piece::player_pieces(black_turn) {
            occupied |= b[p];
            my |= b[p];
        }
        for p in Piece::player_pieces(!black_turn) {
            occupied |= b[p];
            their |= b[p];
        }

        let checkers = self.knight_attacks[king_square] & b[their_knight]
            | self.rook_attacks(king_square, occupied) & their_rook_like
            | self.bishop_attacks(king_square, occupied) & their_bishop_like
            | Self::pawn_move_left(b[king], white_turn) & b[their_pawn]
            | Self::pawn_move_right(b[king], white_turn) & b[their_pawn];

        if checkers.none() {
            return false;
        }

        let empty = !occupied;
        let empty_king = empty | b[king];

        let their_king_square = b[Piece::player_king(white_turn)].first_piece();
        let mut attacked = self.king_attacks[their_king_square];

        for p in b[Piece::player_knight(white_turn)].iter() {
            attacked |= self.knight_attacks[p];
        }

        attacked |= fill_7::nw(their_bishop_like, empty_king);
        attacked |= fill_7::ne(their_bishop_like, empty_king);
        attacked |= fill_7::sw(their_bishop_like, empty_king);
        attacked |= fill_7::se(their_bishop_like, empty_king);
        attacked |= fill_7::n(their_rook_like, empty_king);
        attacked |= fill_7::e(their_rook_like, empty_king);
        attacked |= fill_7::s(their_rook_like, empty_king);
        attacked |= fill_7::w(their_rook_like, empty_king);

        attacked |= Self::pawn_move_left(b[their_pawn], black_turn);
        attacked |= Self::pawn_move_right(b[their_pawn], black_turn);

        if ((empty | checkers) & self.king_attacks[king_square] & !attacked).any() {
            return false;
        }

        if checkers.count() > 1 {
            return true;
        }

        let mut attacked = BB::empty();
        for p in b[their_knight].iter() {
            attacked |= self.knight_attacks[p];
        }

        let rook_pinners = self.xray_rook_attacks(king_square, my, occupied) & their_rook_like;
        let bishop_pinners =
            self.xray_bishop_attacks(king_square, my, occupied) & their_bishop_like;

        let mut pinned = BB::empty();
        for p in rook_pinners.iter() {
            let between = self.between[king_square][p];
            pinned |= my & between;
        }

        for p in bishop_pinners.iter() {
            let between = self.between[king_square][p];
            pinned |= my & between;
        }

        let free_pawns = b[pawn] & !pinned;

        let pawn_attack_checker = Self::pawn_move_left(free_pawns, white_turn)
            | Self::pawn_move_right(free_pawns, white_turn) & checkers;
        if pawn_attack_checker.any() {
            return false;
        }

        let between = self.between[king_square][checkers.first_piece()];

        let pawn_move = Self::pawn_move(free_pawns, white_turn);

        if (pawn_move & between).any() {
            return false;
        }

        let double_move_rank = if white_turn { BB::RANK_3 } else { BB::RANK_6 };

        if (Self::pawn_move(double_move_rank & pawn_move & empty, white_turn) & between).any() {
            return false;
        }

        let block = checkers | between;

        let bishop_like = (b[queen] | b[bishop]) & !pinned;
        let rook_like = (b[queen] | b[rook]) & !pinned;

        if (fill_7::nw(bishop_like, empty) & block).any()
            || (fill_7::ne(bishop_like, empty) & block).any()
            || (fill_7::sw(bishop_like, empty) & block).any()
            || (fill_7::se(bishop_like, empty) & block).any()
            || (fill_7::n(rook_like, empty) & block).any()
            || (fill_7::e(rook_like, empty) & block).any()
            || (fill_7::s(rook_like, empty) & block).any()
            || (fill_7::w(rook_like, empty) & block).any()
        {
            return false;
        }

        for p in b[knight].iter() {
            if (self.knight_attacks[p] & block).any() {
                return false;
            }
        }

        true
    }

    pub fn gen_moves<B: MoveBuffer>(&self, b: &Board, buffer: &mut B) -> bool {
        let black_turn = b.state.black_turn;
        let white_turn = !b.state.black_turn;

        // First generating basic bit boards used repeatedly;
        let mut occupied = BB::empty();
        let mut my = BB::empty();
        let mut their = BB::empty();

        for p in Piece::player_pieces(black_turn) {
            occupied |= b[p];
            my |= b[p];
        }
        for p in Piece::player_pieces(!black_turn) {
            occupied |= b[p];
            their |= b[p];
        }
        let empty = !occupied;

        let their_queen = Piece::player_queen(white_turn);
        let their_pawn = Piece::player_pawn(white_turn);
        let their_bishop = Piece::player_bishop(white_turn);
        let their_rook = Piece::player_rook(white_turn);
        let their_knight = Piece::player_knight(white_turn);
        let their_rook_like = b[their_queen] | b[their_rook];
        let their_bishop_like = b[their_queen] | b[their_bishop];

        let king = Piece::player_king(black_turn);
        let rook = Piece::player_rook(black_turn);
        let bishop = Piece::player_bishop(black_turn);
        let knight = Piece::player_knight(black_turn);
        let queen = Piece::player_queen(black_turn);
        let pawn = Piece::player_pawn(black_turn);

        let bishop_like = b[bishop] | b[queen];
        let rook_like = b[rook] | b[queen];

        let king_square = b[king].first_piece();

        let promote_rank = if white_turn { BB::RANK_7 } else { BB::RANK_2 };

        // Generate bitboard containing attacked pieces;
        let their_king_square = b[Piece::player_king(white_turn)].first_piece();
        let mut attacked = self.king_attacks[their_king_square];

        for p in b[Piece::player_knight(white_turn)].iter() {
            attacked |= self.knight_attacks[p];
        }

        attacked |= fill_7::nw(their_bishop_like, empty);
        attacked |= fill_7::ne(their_bishop_like, empty);
        attacked |= fill_7::sw(their_bishop_like, empty);
        attacked |= fill_7::se(their_bishop_like, empty);
        attacked |= fill_7::n(their_rook_like, empty);
        attacked |= fill_7::e(their_rook_like, empty);
        attacked |= fill_7::s(their_rook_like, empty);
        attacked |= fill_7::w(their_rook_like, empty);

        attacked |= Self::pawn_move_left(b[their_pawn], black_turn);
        attacked |= Self::pawn_move_right(b[their_pawn], black_turn);

        // Generate bitboard containing pinned pieces and possible pinners
        let rook_pinners = self.xray_rook_attacks(king_square, my, occupied) & their_rook_like;
        let bishop_pinners =
            self.xray_bishop_attacks(king_square, my, occupied) & their_bishop_like;

        let mut rook_pinned = BB::empty();
        for p in rook_pinners.iter() {
            let between = self.between[king_square][p];
            rook_pinned |= my & between;
        }

        let mut bishop_pinned = BB::empty();
        for p in bishop_pinners.iter() {
            let between = self.between[king_square][p];
            bishop_pinned |= my & between;
        }

        let pinned = rook_pinned | bishop_pinned;

        // Is the king in check? if so use a different move generation function
        if (b[king] & attacked).any() {
            // update attacked to include squares attacked throught the king
            let king_empty = empty ^ b[king];
            let old_attacked = attacked;
            attacked |= fill_7::nw(their_bishop_like, king_empty);
            attacked |= fill_7::ne(their_bishop_like, king_empty);
            attacked |= fill_7::sw(their_bishop_like, king_empty);
            attacked |= fill_7::se(their_bishop_like, king_empty);
            attacked |= fill_7::n(their_rook_like, king_empty);
            attacked |= fill_7::e(their_rook_like, king_empty);
            attacked |= fill_7::s(their_rook_like, king_empty);
            attacked |= fill_7::w(their_rook_like, king_empty);

            let left_pawn_attack: i8 = if white_turn { 7 } else { -7 };

            let right_pawn_attack: i8 = if white_turn { 9 } else { -9 };

            let checkers = self.knight_attacks[king_square] & b[their_knight]
                | self.rook_attacks(king_square, occupied) & their_rook_like
                | self.bishop_attacks(king_square, occupied) & their_bishop_like
                | Self::pawn_move_left(b[king], white_turn) & b[their_pawn]
                | Self::pawn_move_right(b[king], white_turn) & b[their_pawn];

            // generate king moves as the king can always try to move
            let king_moves = self.king_attacks[king_square] & !attacked & !my;
            for to in (king_moves).iter() {
                buffer.push(Move::normal(king_square, to))
            }

            // multiple checkers, king can has to move, can't block or take
            if checkers.count() > 1 {
                return true;
            }

            if checkers.count() == 0 {
                dbg!(attacked);
                dbg!(old_attacked);
                dbg!(checkers);
                dbg!(b[king]);
            }

            debug_assert_eq!(checkers.count(), 1, "{:?}", b);

            // only one checker
            let checker = checkers.first_piece();
            let mut checker_piece = Piece::WhiteKing;
            for p in their_queen.to(their_pawn) {
                if (b[p] & checkers).any() {
                    checker_piece = p;
                    break;
                }
            }
            debug_assert_ne!(checker_piece, Piece::WhiteKing);

            for k in (b[knight] & !pinned).iter() {
                let attack = self.knight_attacks[k] & checkers;
                if attack.any() {
                    buffer.push(Move::normal(k, attack.first_piece()));
                }
            }

            // none of the pinned pieces can move so always exclude pinned pieces
            let checker_attacked_bishop = self.bishop_attacks(checker, occupied);
            for p in (checker_attacked_bishop & bishop_like & !pinned).iter() {
                buffer.push(Move::normal(p, checker))
            }

            let checker_attacked_rook = self.rook_attacks(checker, occupied);
            for p in (checker_attacked_rook & rook_like & !pinned).iter() {
                buffer.push(Move::normal(p, checker))
            }

            if (Self::pawn_move_left(b[pawn] & !pinned, white_turn) & checkers).any() {
                buffer.push(Move::normal(checker - left_pawn_attack, checker))
            }
            if (Self::pawn_move_right(b[pawn] & !pinned, white_turn) & checkers).any() {
                buffer.push(Move::normal(checker - right_pawn_attack, checker))
            }

            // checks with a knight or pawn can only be resolved by moving or by taking the
            // checking piece
            if checker_piece == their_knight || checker_piece == their_pawn {
                return true;
            }

            // generate moves which put something between checker and the king
            let between = self.between[checker][king_square];
            let pawn_move: i8 = if white_turn { 8 } else { -8 };

            let pawn_moves = Self::pawn_move(b[pawn] & !pinned, white_turn) & empty;
            let double_pawn_moves = Self::pawn_move(pawn_moves & promote_rank, white_turn) & empty;
            for p in (pawn_moves & between).iter() {
                buffer.push(Move::normal(p - pawn_move, p));
            }
            for p in (double_pawn_moves & between).iter() {
                buffer.push(Move::normal(p - pawn_move * 2, p));
            }

            for block in between.iter() {
                let block_attacked_bishop = self.bishop_attacks(block, occupied);
                for p in (block_attacked_bishop & bishop_like & !pinned).iter() {
                    buffer.push(Move::normal(p, block));
                }

                let block_attacked_rook = self.rook_attacks(block, occupied);
                for p in (block_attacked_rook & rook_like & !pinned).iter() {
                    buffer.push(Move::normal(p, block));
                }

                let block_attacked_knight = self.knight_attacks[block] & b[knight] & !pinned;
                for p in block_attacked_knight.iter() {
                    buffer.push(Move::normal(p, block));
                }
            }

            return true;
        }

        // King is not in check. Generate normal moves

        // generate moves for sliding pieces

        // generate bishop moves
        for p in (bishop_like & !pinned).iter() {
            let moves = self.bishop_attacks(p, occupied) & !my;

            for to in moves.iter() {
                buffer.push(Move::normal(p, to));
            }
        }

        // generate moves for pieces pinned by bishop like pieces
        for pinner in bishop_pinners.iter() {
            let between = self.between[king_square][pinner];
            let empty_between = between & empty;
            let pinned = between & bishop_like;
            if (pinned).any() {
                let from = pinned.first_piece();
                for to in empty_between.iter() {
                    buffer.push(Move::normal(from, to));
                }
                buffer.push(Move::normal(from, pinner));
            }
        }

        // generate rook moves
        for p in (rook_like & !pinned).iter() {
            let moves = self.rook_attacks(p, occupied) & !my;

            for to in moves.iter() {
                buffer.push(Move::normal(p, to));
            }
        }

        // generate moves for pieces pinned by bishop like pieces
        for pinner in rook_pinners.iter() {
            let between = self.between[king_square][pinner];
            let empty_between = between & empty;
            let pinned = between & rook_like;
            if (pinned).any() {
                let from = pinned.first_piece();
                for to in empty_between.iter() {
                    buffer.push(Move::normal(from, to));
                }
                buffer.push(Move::normal(from, pinner));
            }
        }

        // generate knight moves
        for p in (b[knight] & !pinned).iter() {
            let moves = self.knight_attacks[p] & !my;
            for to in moves.iter() {
                buffer.push(Move::normal(p, to));
            }
        }

        // generate castle moves
        {
            let from = if black_turn { Square::E8 } else { Square::E1 };
            let castle_flag_shift = if black_turn { 2 } else { 0 };
            let castle_shift = if black_turn { 8 * 7 } else { 0 };
            let king_castle = ExtraState::WHITE_KING_CASTLE << castle_flag_shift;
            let queen_castle = ExtraState::WHITE_QUEEN_CASTLE << castle_flag_shift;
            if b.state.castle & king_castle != 0 {
                let empty = occupied & (Self::CASTLE_KING_EMPTY_MASK << castle_shift);
                let attacked = attacked & (Self::CASTLE_KING_ATTACKED_MASK << castle_shift);
                if !((empty | attacked).any()) {
                    let to = if black_turn { Square::G8 } else { Square::G1 };
                    buffer.push(Move::castle(from, to));
                }
            }
            if b.state.castle & queen_castle != 0 {
                let empty = occupied & (Self::CASTLE_QUEEN_EMPTY_MASK << castle_shift);
                let attacked = attacked & (Self::CASTLE_QUEEN_ATTACKED_MASK << castle_shift);
                if !((empty | attacked).any()) {
                    let to = if black_turn { Square::C8 } else { Square::C1 };
                    buffer.push(Move::castle(from, to));
                }
            }
        }

        // generate king moves
        let king_moves = self.king_attacks[king_square] & !attacked & !my;
        for to in (king_moves).iter() {
            buffer.push(Move::normal(king_square, to));
        }

        // Generate pawn moves
        let pawn = Piece::player_pawn(black_turn);
        let pawns = b[pawn];
        // the rank from which a pawn make a move which causes it to promote
        let last_rank = if white_turn { BB::RANK_8 } else { BB::RANK_1 };
        let free_pawns = pawns & !pinned;

        {
            // first the left side attacks
            let left_pawn_move: i8 = if white_turn { 7 } else { -7 };
            let left_pawn_attacks = Self::pawn_move_left(free_pawns, white_turn) & their;
            // filter out pieces which promote
            for p in (left_pawn_attacks & !last_rank).iter() {
                buffer.push(Move::normal(p - left_pawn_move, p));
            }
            for p in (left_pawn_attacks & last_rank).iter() {
                buffer.push(Move::promotion(
                    p - left_pawn_move,
                    p,
                    Move::PROMOTION_QUEEN,
                ));
                buffer.push(Move::promotion(
                    p - left_pawn_move,
                    p,
                    Move::PROMOTION_KNIGHT,
                ));
                buffer.push(Move::promotion(p - left_pawn_move, p, Move::PROMOTION_ROOK));
                buffer.push(Move::promotion(
                    p - left_pawn_move,
                    p,
                    Move::PROMOTION_BISHOP,
                ));
            }

            // attacks to the right of the pawn
            let right_pawn_move: i8 = if white_turn { 9 } else { -9 };
            let right_pawn_attacks = Self::pawn_move_right(free_pawns, white_turn) & their;
            // filter out pieces which promote
            for p in (right_pawn_attacks & !last_rank).iter() {
                buffer.push(Move::normal(p - right_pawn_move, p));
            }
            for p in (right_pawn_attacks & last_rank).iter() {
                buffer.push(Move::promotion(
                    p - right_pawn_move,
                    p,
                    Move::PROMOTION_QUEEN,
                ));
                buffer.push(Move::promotion(
                    p - right_pawn_move,
                    p,
                    Move::PROMOTION_KNIGHT,
                ));
                buffer.push(Move::promotion(
                    p - right_pawn_move,
                    p,
                    Move::PROMOTION_ROOK,
                ));
                buffer.push(Move::promotion(
                    p - right_pawn_move,
                    p,
                    Move::PROMOTION_BISHOP,
                ));
            }

            //normal pawn advances

            // Pawns which are 'pinned' by a rook in front of it are not really pinned
            // So include those pawns while generating pawn advances
            let mut free_pinned_pawns = BB::empty();
            for p in (pawns & rook_pinned).iter() {
                let move_mask = (BB::FILE_A << p.file() & rook_pinners).saturate();
                free_pinned_pawns |= BB::square(p) & move_mask;
            }

            let free_advance_pawns = free_pinned_pawns | free_pawns;
            let pawn_advance = Self::pawn_move(free_advance_pawns, white_turn) & empty;
            let pawn_move: i8 = if white_turn { 8 } else { -8 };
            let double_move_rank = if white_turn { BB::RANK_3 } else { BB::RANK_6 };
            let pawn_double_advance =
                Self::pawn_move(pawn_advance & double_move_rank, white_turn) & empty;
            // filter out promoted pieces
            let promated_advance = pawn_advance & last_rank;
            let pawn_advance = pawn_advance & !last_rank;

            for p in pawn_advance.iter() {
                buffer.push(Move::normal(p - pawn_move, p));
            }
            for p in pawn_double_advance.iter() {
                buffer.push(Move::normal(p - pawn_move * 2, p));
            }
            for p in promated_advance.iter() {
                buffer.push(Move::promotion(p - pawn_move, p, Move::PROMOTION_QUEEN));
                buffer.push(Move::promotion(p - pawn_move, p, Move::PROMOTION_KNIGHT));
                buffer.push(Move::promotion(p - pawn_move, p, Move::PROMOTION_ROOK));
                buffer.push(Move::promotion(p - pawn_move, p, Move::PROMOTION_BISHOP));
            }

            // moves for pawns which are pinned
            // pawns pinned by rooks are already included in move generation

            // pawns which are pinned by bishops can only move when taking the bishop
            let left_attack_pinned =
                Self::pawn_move_left(pawns & bishop_pinned, white_turn) & bishop_pinners;
            let right_attack_pinned =
                Self::pawn_move_right(pawns & bishop_pinned, white_turn) & bishop_pinners;

            if left_attack_pinned.any() {
                let to = left_attack_pinned.first_piece();
                buffer.push(Move::normal(to - left_pawn_move, to));
            }
            if right_attack_pinned.any() {
                let to = right_attack_pinned.first_piece();
                buffer.push(Move::normal(to - right_pawn_move, to));
            }
        }
        false
    }

    #[inline(always)]
    fn pawn_move_left(b: BB, white_turn: bool) -> BB {
        if white_turn {
            (b & !BB::FILE_A) << 7
        } else {
            (b & !BB::FILE_H) >> 7
        }
    }

    #[inline(always)]
    fn pawn_move_right(b: BB, white_turn: bool) -> BB {
        if white_turn {
            (b & !BB::FILE_H) << 9
        } else {
            (b & !BB::FILE_A) >> 9
        }
    }

    #[inline(always)]
    fn pawn_move(b: BB, white_turn: bool) -> BB {
        if white_turn {
            b << 8
        } else {
            b >> 8
        }
    }
}
