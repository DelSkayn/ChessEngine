#![allow(dead_code)]

use super::util::{BoardArray, DirectionArray};
use super::{Board, Direction, ExtraState, Move, Piece, Player, Square, BB};
use std::mem::MaybeUninit;

mod magic;
use magic::Magic;
pub mod fill_7;
mod types;
pub use types::*;

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

pub trait MoveGenType {
    const CAPTURE: bool;
}

pub struct InlineIter<'a> {
    len: usize,
    cur: usize,
    v: &'a [MaybeUninit<Move>; 256],
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
    magic: Magic,
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
            magic: Magic::new(),
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
            let d = Direction::from_u8(d as u8);
            for i in 0..64 {
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
        /*
        self.ray_attacks_positive(square, occupied, Direction::N)
            | self.ray_attacks_positive(square, occupied, Direction::E)
            | self.ray_attacks_negative(square, occupied, Direction::S)
            | self.ray_attacks_negative(square, occupied, Direction::W)
            */
        self.magic.rook_attacks(square, occupied)
    }

    fn bishop_attacks(&self, square: Square, occupied: BB) -> BB {
        /*
        self.ray_attacks_positive(square, occupied, Direction::NW)
            | self.ray_attacks_positive(square, occupied, Direction::NE)
            | self.ray_attacks_negative(square, occupied, Direction::SE)
            | self.ray_attacks_negative(square, occupied, Direction::SW)
            */
        self.magic.bishop_attacks(square, occupied)
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
        match b.state.player {
            Player::White => self.check_mate_player::<White>(b),
            Player::Black => self.check_mate_player::<Black>(b),
        }
    }

    pub fn check_mate_player<P: PlayerType>(&self, b: &Board) -> bool {
        let their_rook_like = b[P::Opponent::QUEEN] | b[P::Opponent::ROOK];
        let their_bishop_like = b[P::Opponent::QUEEN] | b[P::Opponent::BISHOP];

        let mut occupied = BB::empty();
        let mut my = BB::empty();
        let mut their = BB::empty();

        for p in P::KING.to(P::PAWN) {
            occupied |= b[p];
            my |= b[p];
        }
        for p in P::Opponent::KING.to(P::Opponent::PAWN) {
            occupied |= b[p];
            their |= b[p];
        }

        debug_assert!(b[P::KING].count() > 0);
        let king_square = b[P::KING].first_piece();

        let checkers = self.knight_attacks[king_square] & b[P::Opponent::KNIGHT]
            | self.rook_attacks(king_square, occupied) & their_rook_like
            | self.bishop_attacks(king_square, occupied) & their_bishop_like
            | b[P::KING].shift(P::ATTACK_LEFT) & b[P::Opponent::PAWN]
            | b[P::KING].shift(P::ATTACK_RIGHT) & b[P::Opponent::PAWN];

        if checkers.none() {
            return false;
        }

        let empty = !occupied;
        let empty_king = empty | b[P::KING];

        let their_king_square = b[P::Opponent::KING].first_piece();
        let mut attacked = self.king_attacks[their_king_square];

        for p in b[P::Opponent::KNIGHT].iter() {
            attacked |= self.knight_attacks[p];
        }

        for p in their_bishop_like.iter() {
            attacked |= self.bishop_attacks(p, empty_king)
        }
        for p in their_rook_like.iter() {
            attacked |= self.rook_attacks(p, empty_king)
        }

        attacked |= b[P::Opponent::PAWN].shift(P::Opponent::ATTACK_LEFT);
        attacked |= b[P::Opponent::PAWN].shift(P::Opponent::ATTACK_RIGHT);

        if ((empty | checkers) & self.king_attacks[king_square] & !attacked).any() {
            return false;
        }

        if checkers.count() > 1 {
            return true;
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

        let free_pawns = b[P::PAWN] & !pinned;

        let pawn_attack_checker =
            free_pawns.shift(P::ATTACK_LEFT) | free_pawns.shift(P::ATTACK_RIGHT) & checkers;
        if pawn_attack_checker.any() {
            return false;
        }

        let between = self.between[king_square][checkers.first_piece()];

        let pawn_move = free_pawns.shift(P::PAWN_MOVE);

        if (pawn_move & between).any() {
            return false;
        }

        if ((pawn_move & P::RANK_3 & empty).shift(P::PAWN_MOVE) & between).any() {
            return false;
        }

        let block = checkers | between;

        let bishop_like = b[P::BISHOP] | b[P::QUEEN];
        let rook_like = b[P::ROOK] | b[P::QUEEN];

        for p in bishop_like.iter() {
            if (self.bishop_attacks(p, occupied) & block).any() {
                return false;
            }
        }

        for p in rook_like.iter() {
            if (self.rook_attacks(p, occupied) & block).any() {
                return false;
            }
        }

        for p in b[P::KNIGHT].iter() {
            if (self.knight_attacks[p] & block).any() {
                return false;
            }
        }

        true
    }

    pub fn gen_moves<B: MoveBuffer>(&self, b: &Board, buffer: &mut B) -> bool {
        match b.state.player {
            Player::White => self.gen_moves_player::<B, White>(b, buffer),
            Player::Black => self.gen_moves_player::<B, Black>(b, buffer),
        }
    }

    pub fn gen_attacked<P: PlayerType>(
        &self,
        b: &Board,
        their_rooks: BB,
        their_bishops: BB,
        occupied: BB,
    ) -> BB {
        let mut attacked = self.king_attacks[b[P::Opponent::KING].first_piece()];
        for p in b[P::Opponent::KNIGHT].iter() {
            attacked |= self.knight_attacks[p];
        }

        for p in their_bishops.iter() {
            attacked |= self.magic.bishop_attacks(p, occupied);
        }
        for p in their_rooks.iter() {
            attacked |= self.magic.rook_attacks(p, occupied);
        }

        attacked |= b[P::Opponent::PAWN].shift(P::Opponent::ATTACK_LEFT);
        attacked |= b[P::Opponent::PAWN].shift(P::Opponent::ATTACK_RIGHT);

        attacked
    }

    pub fn gen_moves_player<B: MoveBuffer, P: PlayerType>(
        &self,
        b: &Board,
        buffer: &mut B,
    ) -> bool {
        assert!(b.state.player == P::VALUE);
        // First generating basic bit boards used repeatedly;
        let mut occupied = BB::empty();
        let mut my = BB::empty();
        let mut their = BB::empty();

        for p in P::KING.to(P::PAWN) {
            occupied |= b[p];
            my |= b[p];
        }
        for p in P::Opponent::KING.to(P::Opponent::PAWN) {
            occupied |= b[p];
            their |= b[p];
        }
        let empty = !occupied;

        let bishop_like = b[P::BISHOP] | b[P::QUEEN];
        let rook_like = b[P::ROOK] | b[P::QUEEN];

        let their_bishop_like = b[P::Opponent::BISHOP] | b[P::Opponent::QUEEN];
        let their_rook_like = b[P::Opponent::ROOK] | b[P::Opponent::QUEEN];

        let king_square = b[P::KING].first_piece();

        // Generate bitboard containing attacked pieces;
        let mut attacked = self.gen_attacked::<P>(b, their_rook_like, their_bishop_like, occupied);

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
        if (b[P::KING] & attacked).any() {
            // update attacked to include squares attacked throught the king
            let occ_no_king = occupied ^ b[P::KING];
            let old_attacked = attacked;
            for p in their_bishop_like.iter() {
                attacked |= self.bishop_attacks(p, occ_no_king);
            }
            for p in their_rook_like.iter() {
                attacked |= self.rook_attacks(p, occ_no_king);
            }

            let checkers = self.knight_attacks[king_square] & b[P::Opponent::KNIGHT]
                | self.rook_attacks(king_square, occupied) & their_rook_like
                | self.bishop_attacks(king_square, occupied) & their_bishop_like
                | b[P::KING].shift(P::ATTACK_LEFT) & b[P::Opponent::PAWN]
                | b[P::KING].shift(P::ATTACK_RIGHT) & b[P::Opponent::PAWN];

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
                dbg!(their_rook_like);
                dbg!(their_bishop_like);
                dbg!(occupied);
                dbg!(self.rook_attacks(king_square, occupied));
                dbg!(self.bishop_attacks(king_square, occupied));
                dbg!(attacked);
                dbg!(old_attacked);
                dbg!(checkers);
                dbg!(b[P::KING]);
                panic!()
            }

            debug_assert_eq!(checkers.count(), 1, "{:?}", b);

            // only one checker
            let checker = checkers.first_piece();
            let mut checker_piece = Piece::WhiteKing;
            for p in P::Opponent::QUEEN.to(P::Opponent::PAWN) {
                if (b[p] & checkers).any() {
                    checker_piece = p;
                    break;
                }
            }
            debug_assert_ne!(checker_piece, Piece::WhiteKing);

            for k in (b[P::KNIGHT] & !pinned).iter() {
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

            let free_pawns = b[P::PAWN] & !pinned;
            if (free_pawns.shift(P::ATTACK_LEFT) & checkers).any() {
                buffer.push(Move::normal(checker - P::ATTACK_LEFT.as_offset(), checker))
            }
            if (free_pawns.shift(P::ATTACK_RIGHT) & checkers).any() {
                buffer.push(Move::normal(checker - P::ATTACK_RIGHT.as_offset(), checker))
            }

            // checks with a knight pawn can only be resolved by moving the king or by taking the
            // checking piece
            if checker_piece == P::Opponent::KNIGHT || checker_piece == P::Opponent::PAWN {
                return true;
            }

            // generate moves which put something between checker and the king
            let between = self.between[checker][king_square];

            let pawn_moves = free_pawns.shift(P::PAWN_MOVE) & empty;
            let double_pawn_moves = (pawn_moves & P::RANK_3).shift(P::PAWN_MOVE) & empty;
            for p in (pawn_moves & between).iter() {
                buffer.push(Move::normal(p - P::PAWN_MOVE.as_offset(), p));
            }
            for p in (double_pawn_moves & between).iter() {
                buffer.push(Move::normal(p - P::PAWN_MOVE.as_offset() * 2, p));
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

                let block_attacked_knight = self.knight_attacks[block] & b[P::KNIGHT] & !pinned;
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
        for p in (b[P::KNIGHT] & !pinned).iter() {
            let moves = self.knight_attacks[p] & !my;
            for to in moves.iter() {
                buffer.push(Move::normal(p, to));
            }
        }

        // generate castle moves
        {
            let from = P::CASTLE_FROM;
            let king_castle = ExtraState::WHITE_KING_CASTLE << P::FLAG_SHIFT;
            let queen_castle = ExtraState::WHITE_QUEEN_CASTLE << P::FLAG_SHIFT;
            if b.state.castle & king_castle != 0 {
                let empty = occupied & (Self::CASTLE_KING_EMPTY_MASK << P::CASTLE_SHIFT);
                let attacked = attacked & (Self::CASTLE_KING_ATTACKED_MASK << P::CASTLE_SHIFT);
                if !((empty | attacked).any()) {
                    let to = P::CASTLE_KING_TO;
                    buffer.push(Move::castle(from, to));
                }
            }
            if b.state.castle & queen_castle != 0 {
                let empty = occupied & (Self::CASTLE_QUEEN_EMPTY_MASK << P::CASTLE_SHIFT);
                let attacked = attacked & (Self::CASTLE_QUEEN_ATTACKED_MASK << P::CASTLE_SHIFT);
                if !((empty | attacked).any()) {
                    let to = P::CASTLE_QUEEN_TO;
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
        // the rank from which a pawn make a move which causes it to promote
        {
            let free_pawns = b[P::PAWN] & !pinned;
            // first the left side attacks
            let left_pawn_attacks = free_pawns.shift(P::ATTACK_LEFT) & their;
            // filter out pieces which promote
            for p in (left_pawn_attacks & !P::RANK_8).iter() {
                buffer.push(Move::normal(p - P::ATTACK_LEFT.as_offset(), p));
            }
            for p in (left_pawn_attacks & P::RANK_8).iter() {
                let from = p - P::ATTACK_LEFT.as_offset();
                buffer.push(Move::promotion(from, p, Move::PROMOTION_QUEEN));
                buffer.push(Move::promotion(from, p, Move::PROMOTION_KNIGHT));
                buffer.push(Move::promotion(from, p, Move::PROMOTION_ROOK));
                buffer.push(Move::promotion(from, p, Move::PROMOTION_BISHOP));
            }

            // attacks to the right of the pawn
            let right_pawn_attacks = free_pawns.shift(P::ATTACK_RIGHT) & their;
            // filter out pieces which promote
            for p in (right_pawn_attacks & !P::RANK_8).iter() {
                buffer.push(Move::normal(p - P::ATTACK_RIGHT.as_offset(), p));
            }
            for p in (right_pawn_attacks & P::RANK_8).iter() {
                let from = p - P::ATTACK_RIGHT.as_offset();
                buffer.push(Move::promotion(from, p, Move::PROMOTION_QUEEN));
                buffer.push(Move::promotion(from, p, Move::PROMOTION_KNIGHT));
                buffer.push(Move::promotion(from, p, Move::PROMOTION_ROOK));
                buffer.push(Move::promotion(from, p, Move::PROMOTION_BISHOP));
            }

            //normal pawn advances

            // Pawns which are 'pinned' by a rook in front of it are not really pinned
            // So include those pawns while generating pawn advances
            let mut free_pinned_pawns = BB::empty();
            for p in (b[P::PAWN] & rook_pinned).iter() {
                let move_mask = (BB::FILE_A << p.file() & rook_pinners).saturate();
                free_pinned_pawns |= BB::square(p) & move_mask;
            }

            let free_advance_pawns = free_pinned_pawns | free_pawns;
            let pawn_advance = free_advance_pawns.shift(P::PAWN_MOVE) & empty;
            let pawn_double_advance = (pawn_advance & P::RANK_3).shift(P::PAWN_MOVE) & empty;
            // filter out promoted pieces
            let promated_advance = pawn_advance & P::RANK_8;
            let pawn_advance = pawn_advance & !P::RANK_8;

            for p in pawn_advance.iter() {
                buffer.push(Move::normal(p - P::PAWN_MOVE.as_offset(), p));
            }
            for p in pawn_double_advance.iter() {
                buffer.push(Move::normal(p - P::PAWN_MOVE.as_offset() * 2, p));
            }
            for p in promated_advance.iter() {
                let from = p - P::PAWN_MOVE.as_offset();
                buffer.push(Move::promotion(from, p, Move::PROMOTION_QUEEN));
                buffer.push(Move::promotion(from, p, Move::PROMOTION_KNIGHT));
                buffer.push(Move::promotion(from, p, Move::PROMOTION_ROOK));
                buffer.push(Move::promotion(from, p, Move::PROMOTION_BISHOP));
            }

            // moves for pawns which are pinned
            // pawns pinned by rooks are already included in move generation

            // pawns which are pinned by bishops can only move when taking the bishop
            let left_attack_pinned = b[P::PAWN].shift(P::ATTACK_LEFT) & bishop_pinners;
            let right_attack_pinned = b[P::PAWN].shift(P::ATTACK_RIGHT) & bishop_pinners;

            if left_attack_pinned.any() {
                let to = left_attack_pinned.first_piece();
                buffer.push(Move::normal(to - P::ATTACK_LEFT.as_offset(), to));
            }
            if right_attack_pinned.any() {
                let to = right_attack_pinned.first_piece();
                buffer.push(Move::normal(to - P::ATTACK_RIGHT.as_offset(), to));
            }
        }
        false
    }
}
