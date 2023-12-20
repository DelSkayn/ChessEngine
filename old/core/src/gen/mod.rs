//! Move generation routines.

use crate::{
    bb::BB,
    board::{Board, MoveChain},
    mov::Promotion,
    ExtraState, Move, Piece, Square,
};

pub mod fill_7;

mod types;
pub use types::*;

mod tables;
use tables::Tables;

use std::mem::MaybeUninit;

pub const MAX_MOVES: usize = 218;

/// A constant size buffer stored on the stack,
/// Can be used for storing moves without allocation.
#[derive(Copy)]
pub struct InlineBuffer<const SIZE: usize = MAX_MOVES, T: Copy = Move> {
    moves: [MaybeUninit<T>; SIZE],
    len: u16,
}

impl<const SIZE: usize, T: Copy> Clone for InlineBuffer<SIZE, T> {
    fn clone(&self) -> Self {
        // TODO: Maybe remove copy?
        *self
    }
}

impl<const SIZE: usize, T: Copy> InlineBuffer<SIZE, T> {
    #[inline]
    pub fn new() -> Self {
        debug_assert!(SIZE <= u16::MAX.into());
        InlineBuffer {
            moves: [MaybeUninit::uninit(); SIZE],
            len: 0,
        }
    }

    #[inline]
    pub fn iter(&self) -> InlineIter<SIZE, T> {
        InlineIter {
            len: self.len,
            cur: 0,
            v: &self.moves,
        }
    }

    #[inline]
    pub fn swap_remove(&mut self, idx: usize) {
        assert!(
            idx < self.len as usize,
            "got idx: {} while len is {}",
            idx,
            self.len
        );
        self.moves.swap(idx, self.len as usize - 1);
        self.len -= 1;
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.len > 0 {
            self.len -= 1;
            Some(unsafe { self.moves[self.len as usize].assume_init() })
        } else {
            None
        }
    }
}

impl<const SIZE: usize, T: Copy> Default for InlineBuffer<SIZE, T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct InlineIter<'a, const SIZE: usize, T: Copy = Move> {
    len: u16,
    cur: u16,
    v: &'a [MaybeUninit<T>; SIZE],
}

impl<'a, const SIZE: usize, T: Copy> Iterator for InlineIter<'a, SIZE, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == self.cur {
            return None;
        }
        let res = unsafe { *self.v.get_unchecked(self.cur as usize).as_ptr() };
        self.cur += 1;
        Some(res)
    }
}

impl<const SIZE: usize> MoveList for InlineBuffer<SIZE> {
    fn push(&mut self, m: Move) {
        assert!((self.len as usize) < SIZE);
        self.moves[self.len as usize] = MaybeUninit::new(m);
        self.len += 1;
    }

    fn get(&self, idx: usize) -> Move {
        assert!(idx < self.len as usize);
        unsafe { self.moves[idx].assume_init() }
    }

    fn set(&mut self, idx: usize, m: Move) {
        assert!(idx < self.len as usize);
        self.moves[idx] = MaybeUninit::new(m);
    }

    fn clear(&mut self) {
        self.len = 0;
    }

    fn len(&self) -> usize {
        self.len as usize
    }
    fn truncate(&mut self, len: usize) {
        assert!(len <= self.len as usize);
        self.len = len as u16;
    }

    fn swap(&mut self, a: usize, b: usize) {
        self.moves.swap(a, b);
    }
}

/// Trait which a list must implement to be used by the move generator.
pub trait MoveList {
    /// Add a move to the list.
    fn push(&mut self, m: Move);

    /// Return the move at the given position.
    /// Should panic if idx is outside bounds.
    fn get(&self, idx: usize) -> Move;

    /// Set the move of give index.
    /// Should panic if idx is outside bounds.
    fn set(&mut self, idx: usize, m: Move);

    /// Remove all moves.
    fn clear(&mut self);

    /// Return the number of moves.
    fn len(&self) -> usize;

    /// Make the length at maximum the given length.
    fn truncate(&mut self, len: usize);

    /// Swap to moves at given index.
    fn swap(&mut self, a: usize, b: usize);

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl MoveList for Vec<Move> {
    fn push(&mut self, m: Move) {
        self.push(m);
    }

    fn get(&self, idx: usize) -> Move {
        self[idx]
    }

    fn set(&mut self, idx: usize, m: Move) {
        self[idx] = m;
    }

    fn clear(&mut self) {
        self.clear()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn truncate(&mut self, len: usize) {
        self.truncate(len);
    }

    fn swap(&mut self, a: usize, b: usize) {
        (**self).swap(a, b)
    }
}

/// Info about a position used in various move generation functions.
pub struct PositionInfo {
    pub occupied: BB,
    pub my: BB,
    pub their: BB,
    pub attacked: BB,
    pub blockers: BB,
    pub pinners: BB,
}

impl PositionInfo {
    /// Create position info of a given position.
    pub fn about<P: Player, M: MoveChain>(table: Tables, b: &Board<M>) -> Self {
        let their_rooks = b.pieces[P::Opponent::QUEEN] | b.pieces[P::Opponent::ROOK];
        let their_bishops = b.pieces[P::Opponent::QUEEN] | b.pieces[P::Opponent::BISHOP];
        let king_sq = b.pieces[P::KING].first_piece();

        let my = b.pieces[P::KING]
            | b.pieces[P::QUEEN]
            | b.pieces[P::ROOK]
            | b.pieces[P::BISHOP]
            | b.pieces[P::KNIGHT]
            | b.pieces[P::PAWN];

        let their = b.pieces[P::Opponent::KING]
            | b.pieces[P::Opponent::BISHOP]
            | b.pieces[P::Opponent::KNIGHT]
            | b.pieces[P::Opponent::PAWN]
            | their_rooks;

        let occupied = my | their;

        let mut attacked = table.king_attacks(b.pieces[P::Opponent::KING].first_piece());

        for k in b.pieces[P::Opponent::KNIGHT].iter() {
            attacked |= table.knight_attacks(k);
        }

        let bishops = b.pieces[P::Opponent::BISHOP] | b.pieces[P::Opponent::QUEEN];
        for b in bishops.iter() {
            attacked |= table.bishop_attacks(b, occupied);
        }

        let rooks = b.pieces[P::Opponent::ROOK] | b.pieces[P::Opponent::QUEEN];
        for b in rooks.iter() {
            attacked |= table.rook_attacks(b, occupied);
        }

        attacked |= b.pieces[P::Opponent::PAWN].shift(P::Opponent::ATTACK_LEFT);
        attacked |= b.pieces[P::Opponent::PAWN].shift(P::Opponent::ATTACK_RIGHT);

        let mut pinners = Self::xray_rook_attacks(table, king_sq, occupied) & their_rooks;
        pinners |= Self::xray_bishop_attacks(table, king_sq, occupied) & their_bishops;

        let mut blockers = BB::empty();
        for p in pinners {
            blockers |= my & table.between(king_sq, p);
        }

        PositionInfo {
            occupied,
            my,
            their,
            attacked,
            blockers,
            pinners,
        }
    }

    fn xray_rook_attacks(table: Tables, sq: Square, mut occ: BB) -> BB {
        let rook_attacked = table.rook_attacks(sq, occ);
        occ &= !rook_attacked;
        table.rook_attacks(sq, occ)
    }

    fn xray_bishop_attacks(table: Tables, sq: Square, mut occ: BB) -> BB {
        let rook_attacked = table.bishop_attacks(sq, occ);
        occ &= !rook_attacked;
        table.bishop_attacks(sq, occ)
    }
}

/// Move generator.
pub struct MoveGenerator {
    tables: Tables,
}

impl MoveGenerator {
    pub fn new() -> Self {
        MoveGenerator {
            tables: Tables::new(),
        }
    }

    #[inline]
    pub fn gen_moves<T: GenType, M: MoveList, C: MoveChain>(
        &self,
        b: &Board<C>,
        list: &mut M,
    ) -> PositionInfo {
        match b.state.player {
            crate::Player::White => self.gen_moves_player::<White, T, M, C>(b, list),
            crate::Player::Black => self.gen_moves_player::<Black, T, M, C>(b, list),
        }
    }

    #[inline]
    pub fn gen_moves_info<T: GenType, M: MoveList, C: MoveChain>(
        &self,
        b: &Board<C>,
        info: &PositionInfo,
        list: &mut M,
    ) {
        match b.state.player {
            crate::Player::White => self.gen_moves_player_info::<White, T, M, C>(b, info, list),
            crate::Player::Black => self.gen_moves_player_info::<Black, T, M, C>(b, info, list),
        }
    }

    #[inline]
    pub fn gen_info<C: MoveChain>(&self, board: &Board<C>) -> PositionInfo {
        match board.state.player {
            crate::Player::White => PositionInfo::about::<White, C>(self.tables, board),
            crate::Player::Black => PositionInfo::about::<Black, C>(self.tables, board),
        }
    }

    /// Returns wether the position is in checkmate.
    #[inline]
    pub fn is_checkmate<M: MoveChain>(&self, b: &Board<M>, info: &PositionInfo) -> bool {
        match b.state.player {
            crate::Player::White => self.is_checkmate_player::<White, M>(b, info),
            crate::Player::Black => self.is_checkmate_player::<Black, M>(b, info),
        }
    }

    /// Returns wether the king for the current player is in check.
    #[inline]
    pub fn is_king_checked<M: MoveChain>(&self, b: &Board<M>, info: &PositionInfo) -> bool {
        match b.state.player {
            crate::Player::White => self.is_king_checked_player::<White, M>(b, info),
            crate::Player::Black => self.is_king_checked_player::<Black, M>(b, info),
        }
    }

    /// Checks if the board is drawn due lack of material or move_clock
    pub fn is_board_drawn<M: MoveChain>(&self, b: &Board<M>, info: &PositionInfo) -> bool {
        // If 50 moves happend without progress the game is drawn
        if b.state.move_clock == 50 {
            return true;
        }
        let piece_count = info.occupied.count();
        if piece_count > 5 {
            return false;
        }
        if piece_count < 3 {
            return true;
        }
        if (b.pieces[Piece::WhiteRook]
            | b.pieces[Piece::WhitePawn]
            | b.pieces[Piece::BlackRook]
            | b.pieces[Piece::BlackPawn])
            .any()
        {
            return false;
        }
        if piece_count == 3 {
            return true;
        }
        if piece_count == 4 {
            return (b.pieces[Piece::WhiteBishop] | b.pieces[Piece::WhiteKnight]).count() == 1;
        }

        if b.pieces[Piece::WhiteBishop].count() == 2 {
            if (b.pieces[Piece::WhiteBishop] & BB::WHITE_SQUARES).count() == 1 {
                return b.pieces[Piece::BlackBishop].any();
            } else {
                return true;
            }
        }
        if b.pieces[Piece::BlackBishop].count() == 2 {
            if (b.pieces[Piece::BlackBishop] & BB::WHITE_SQUARES).count() == 1 {
                return b.pieces[Piece::WhiteBishop].any();
            } else {
                return true;
            }
        }
        true
    }

    pub fn is_king_checked_player<P: Player, C: MoveChain>(
        &self,
        b: &Board<C>,
        info: &PositionInfo,
    ) -> bool {
        let king_sq = b.pieces[P::KING].first_piece();
        let attackers = self.tables.bishop_attacks(king_sq, info.occupied)
            & (b.pieces[P::Opponent::QUEEN] | b.pieces[P::Opponent::BISHOP])
            | self.tables.rook_attacks(king_sq, info.occupied)
                & (b.pieces[P::Opponent::QUEEN] | b.pieces[P::Opponent::ROOK])
            | self.tables.knight_attacks(king_sq) & b.pieces[P::Opponent::KNIGHT]
            | b.pieces[P::KING].shift(P::ATTACK_LEFT) & b.pieces[P::Opponent::PAWN]
            | b.pieces[P::KING].shift(P::ATTACK_RIGHT) & b.pieces[P::Opponent::PAWN];
        attackers.count() > 0
    }

    pub fn is_checkmate_player<P: Player, C: MoveChain>(
        &self,
        b: &Board<C>,
        info: &PositionInfo,
    ) -> bool {
        let king_sq = b.pieces[P::KING].first_piece();
        let attackers = self.tables.bishop_attacks(king_sq, info.occupied)
            & (b.pieces[P::Opponent::QUEEN] | b.pieces[P::Opponent::BISHOP])
            | self.tables.rook_attacks(king_sq, info.occupied)
                & (b.pieces[P::Opponent::QUEEN] | b.pieces[P::Opponent::ROOK])
            | self.tables.knight_attacks(king_sq) & b.pieces[P::Opponent::KNIGHT]
            | b.pieces[P::KING].shift(P::ATTACK_LEFT) & b.pieces[P::Opponent::PAWN]
            | b.pieces[P::KING].shift(P::ATTACK_RIGHT) & b.pieces[P::Opponent::PAWN];

        let attackers_count = attackers.count();

        if attackers_count == 0 {
            return false;
        }

        let mut king_moves = self.tables.king_attacks(king_sq) & !(info.my | info.attacked);
        for p in attackers & !(b.pieces[P::Opponent::KNIGHT] | b.pieces[P::Opponent::PAWN]) {
            king_moves &= !(self.tables.line(king_sq, p) & !BB::square(p));
        }

        if king_moves.any() {
            return false;
        }

        if attackers_count > 1 {
            return true;
        }

        let mut blockers = attackers;
        if (attackers & b.pieces[P::Opponent::KNIGHT]).none() {
            blockers |= self.tables.between(attackers.first_piece(), king_sq);
        }

        let mut list = InlineBuffer::<128>::new();
        self.gen_moves_sliders::<P, gen_type::All, _, _>(b, info, &mut list, blockers);
        if list.len() > 0 {
            return false;
        }
        list.clear();
        self.gen_moves_knight::<P, _, _>(b, &mut list, blockers);
        if list.len() > 0 {
            return false;
        }
        list.clear();
        self.gen_pawn_moves::<P, _, _>(b, info, &mut list, blockers);
        if list.len() > 0 {
            return false;
        }
        true
    }

    pub fn gen_moves_player<P: Player, T: GenType, M: MoveList, C: MoveChain>(
        &self,
        b: &Board<C>,
        list: &mut M,
    ) -> PositionInfo {
        let info = PositionInfo::about::<P, _>(self.tables, b);
        self.gen_moves_player_info::<P, T, M, C>(b, &info, list);
        info
    }

    #[inline]
    pub fn gen_moves_player_info<P: Player, T: GenType, M: MoveList, C: MoveChain>(
        &self,
        b: &Board<C>,
        info: &PositionInfo,
        list: &mut M,
    ) {
        let target = if T::QUIET { !info.my } else { info.their };

        if (info.attacked & b.pieces[P::KING]).any() {
            self.gen_evasion::<P, T, _, _>(b, info, list, target);
        } else {
            self.gen_moves_pseudo::<P, T, _, _>(b, info, list, target);
        }

        if T::LEGAL {
            let mut cur = 0;
            for i in 0..list.len() {
                let m = list.get(i);
                if self.is_legal_player::<P, _>(list.get(i), b, info) {
                    list.set(cur, m);
                    cur += 1;
                }
            }
            list.truncate(cur);
        }
    }

    pub fn gen_evasion<P: Player, T: GenType, M: MoveList, C: MoveChain>(
        &self,
        b: &Board<C>,
        info: &PositionInfo,
        list: &mut M,
        target: BB,
    ) {
        let king_sq = b.pieces[P::KING].first_piece();
        let attackers = self.tables.bishop_attacks(king_sq, info.occupied)
            & (b.pieces[P::Opponent::QUEEN] | b.pieces[P::Opponent::BISHOP])
            | self.tables.rook_attacks(king_sq, info.occupied)
                & (b.pieces[P::Opponent::QUEEN] | b.pieces[P::Opponent::ROOK])
            | self.tables.knight_attacks(king_sq) & b.pieces[P::Opponent::KNIGHT]
            | b.pieces[P::KING].shift(P::ATTACK_LEFT) & b.pieces[P::Opponent::PAWN]
            | b.pieces[P::KING].shift(P::ATTACK_RIGHT) & b.pieces[P::Opponent::PAWN];

        let attackers_count = attackers.count();

        assert!(attackers_count > 0);

        let mut king_moves = self.tables.king_attacks(king_sq) & target & !info.attacked;
        for p in attackers & !(b.pieces[P::Opponent::KNIGHT] | b.pieces[P::Opponent::PAWN]) {
            king_moves &= !(self.tables.line(king_sq, p) & !BB::square(p));
        }

        for p in king_moves {
            list.push(Move::normal(king_sq, p));
        }

        if attackers_count > 1 {
            return;
        }

        let mut blockers = attackers;
        if T::QUIET && (attackers & b.pieces[P::Opponent::KNIGHT]).none() {
            blockers |= self.tables.between(attackers.first_piece(), king_sq);
        }
        self.gen_moves_sliders::<P, T, M, _>(b, info, list, blockers);
        self.gen_moves_knight::<P, M, _>(b, list, blockers);
        self.gen_pawn_moves::<P, M, _>(b, info, list, blockers);
    }

    pub fn gen_moves_pseudo<P: Player, T: GenType, M: MoveList, C: MoveChain>(
        &self,
        b: &Board<C>,
        info: &PositionInfo,
        list: &mut M,
        target: BB,
    ) {
        let king_sq = b.pieces[P::KING].first_piece();
        for s in self.tables.king_attacks(king_sq) & target {
            list.push(Move::normal(king_sq, s));
        }

        {
            let mut target = target;
            if T::CHECKS {
                target |= b.pieces[P::Opponent::KING].shift(P::Opponent::ATTACK_LEFT)
                    | b.pieces[P::Opponent::KING].shift(P::Opponent::ATTACK_LEFT)
            }
            self.gen_pawn_moves::<P, M, _>(b, info, list, target);
        }
        {
            let mut target = target;
            if T::CHECKS {
                target |= self
                    .tables
                    .knight_attacks(b.pieces[P::Opponent::KING].first_piece())
            }
            self.gen_moves_knight::<P, M, _>(b, list, target);
        }
        self.gen_moves_sliders::<P, T, M, _>(b, info, list, target);
        if T::QUIET {
            self.gen_castle::<P, M, _>(b, info, list);
        }
    }

    pub fn gen_pawn_moves<P: Player, M: MoveList, C: MoveChain>(
        &self,
        b: &Board<C>,
        info: &PositionInfo,
        list: &mut M,
        target: BB,
    ) {
        let tgt = target & !P::RANK_8;
        let tgt_promote = target & P::RANK_8;

        let attacked = b.pieces[P::PAWN].shift(P::ATTACK_LEFT) & info.their;
        for p in attacked & tgt {
            list.push(Move::normal(p - P::ATTACK_LEFT.as_offset(), p));
        }
        for p in attacked & tgt_promote {
            self.gen_promotions(p - P::ATTACK_LEFT.as_offset(), p, list);
        }

        let attacked = b.pieces[P::PAWN].shift(P::ATTACK_RIGHT) & info.their;
        for p in attacked & tgt {
            list.push(Move::normal(p - P::ATTACK_RIGHT.as_offset(), p));
        }
        for p in attacked & tgt_promote {
            self.gen_promotions(p - P::ATTACK_RIGHT.as_offset(), p, list);
        }

        let moved = b.pieces[P::PAWN].shift(P::PAWN_MOVE) & !info.occupied;
        let pawn_move = P::PAWN_MOVE.as_offset();
        for p in moved & tgt {
            list.push(Move::normal(p - pawn_move, p));
        }
        for p in moved & tgt_promote {
            self.gen_promotions(p - pawn_move, p, list);
        }

        let double_moved = (moved & P::RANK_3).shift(P::PAWN_MOVE) & !info.occupied;
        let double_pawn_move = pawn_move + pawn_move;
        for p in double_moved & tgt {
            list.push(Move::double_pawn(p - double_pawn_move, p));
        }

        if b.state.en_passant != ExtraState::INVALID_ENPASSANT {
            let file = BB::FILE_A << b.state.en_passant;
            if (file & (P::Opponent::RANK_3 | P::RANK_5) & target).none() {
                return;
            }
            let pawn_on_rank = b.pieces[P::PAWN] & P::RANK_5;
            let pawn = file.shift(P::LEFT) & pawn_on_rank;
            if pawn.any() {
                let sq = pawn.first_piece();
                list.push(Move::en_passant(
                    pawn.first_piece(),
                    sq + P::ATTACK_RIGHT.as_offset(),
                ));
            }
            let pawn = file.shift(P::RIGHT) & pawn_on_rank;
            if pawn.any() {
                let sq = pawn.first_piece();
                list.push(Move::en_passant(
                    pawn.first_piece(),
                    sq + P::ATTACK_LEFT.as_offset(),
                ));
            }
        }
    }

    pub fn gen_castle<P: Player, M: MoveList, C: MoveChain>(
        &self,
        b: &Board<C>,
        info: &PositionInfo,
        list: &mut M,
    ) {
        const CASTLE_KING_ATTACKED_MASK: BB = BB(0b01100000);
        const CASTLE_QUEEN_ATTACKED_MASK: BB = BB(0b00001100);
        const CASTLE_KING_EMPTY_MASK: BB = BB(0b01100000);
        const CASTLE_QUEEN_EMPTY_MASK: BB = BB(0b00001110);

        let occupied = info.occupied;

        let from = P::CASTLE_FROM;
        let king_castle = ExtraState::WHITE_KING_CASTLE << P::FLAG_SHIFT;
        let queen_castle = ExtraState::WHITE_QUEEN_CASTLE << P::FLAG_SHIFT;
        if b.state.castle & king_castle != 0 {
            let empty = occupied & (CASTLE_KING_EMPTY_MASK << P::CASTLE_SHIFT);
            let attacked = info.attacked & (CASTLE_KING_ATTACKED_MASK << P::CASTLE_SHIFT);
            if !((empty | attacked).any()) {
                let to = P::CASTLE_KING_TO;
                list.push(Move::castle(from, to));
            }
        }
        if b.state.castle & queen_castle != 0 {
            let empty = occupied & (CASTLE_QUEEN_EMPTY_MASK << P::CASTLE_SHIFT);
            let attacked = info.attacked & (CASTLE_QUEEN_ATTACKED_MASK << P::CASTLE_SHIFT);
            if !((empty | attacked).any()) {
                let to = P::CASTLE_QUEEN_TO;
                list.push(Move::castle(from, to));
            }
        }
    }

    pub fn gen_promotions<M: MoveList>(&self, from: Square, to: Square, list: &mut M) {
        list.push(Move::promotion(from, to, Promotion::Queen));
        list.push(Move::promotion(from, to, Promotion::Rook));
        list.push(Move::promotion(from, to, Promotion::Knight));
        list.push(Move::promotion(from, to, Promotion::Bishop));
    }

    pub fn gen_moves_sliders<P: Player, T: GenType, M: MoveList, C: MoveChain>(
        &self,
        b: &Board<C>,
        info: &PositionInfo,
        list: &mut M,
        target: BB,
    ) {
        {
            let mut target = target;
            if T::CHECKS {
                target |= self
                    .tables
                    .bishop_attacks(b.pieces[P::Opponent::KING].first_piece(), info.occupied)
                    & !info.my;
            }

            let bishops = b.pieces[P::BISHOP] | b.pieces[P::QUEEN];
            for b in bishops {
                for m in self.tables.bishop_attacks(b, info.occupied) & target {
                    list.push(Move::normal(b, m));
                }
            }
        }

        {
            let mut target = target;
            if T::CHECKS {
                target |= self
                    .tables
                    .rook_attacks(b.pieces[P::Opponent::KING].first_piece(), info.occupied)
                    & !info.my;
            }
            let rooks = b.pieces[P::ROOK] | b.pieces[P::QUEEN];
            for b in rooks {
                for m in self.tables.rook_attacks(b, info.occupied) & target {
                    list.push(Move::normal(b, m));
                }
            }
        }
    }

    pub fn gen_moves_knight<P: Player, M: MoveList, C: MoveChain>(
        &self,
        b: &Board<C>,
        list: &mut M,
        target: BB,
    ) {
        for k in b.pieces[P::KNIGHT] {
            for a in self.tables.knight_attacks(k) & target {
                list.push(Move::normal(k, a));
            }
        }
    }
    pub fn is_legal<C: MoveChain>(&self, m: Move, b: &Board<C>, info: &PositionInfo) -> bool {
        match b.state.player {
            crate::Player::White => self.is_legal_player::<White, C>(m, b, info),
            crate::Player::Black => self.is_legal_player::<Black, C>(m, b, info),
        }
    }

    pub fn is_legal_player<P: Player, C: MoveChain>(
        &self,
        m: Move,
        b: &Board<C>,
        info: &PositionInfo,
    ) -> bool {
        let from = m.from();
        let to = m.to();

        if m.ty() == Move::TYPE_EN_PASSANT {
            let king_sq = b.pieces[P::KING].first_piece();
            let captured = to - P::PAWN_MOVE.as_offset();
            let occupied =
                info.occupied ^ (BB::square(captured) | BB::square(from) | BB::square(to));

            return (self.tables.bishop_attacks(king_sq, occupied)
                & (b.pieces[P::Opponent::QUEEN] | b.pieces[P::Opponent::BISHOP]))
                .none()
                && (self.tables.rook_attacks(king_sq, occupied)
                    & (b.pieces[P::Opponent::QUEEN] | b.pieces[P::Opponent::ROOK]))
                    .none();
        }

        if Some(P::KING) == b.on(from) {
            return m.ty() == Move::TYPE_CASTLE || (BB::square(m.to()) & info.attacked).none();
        }

        (info.blockers & BB::square(m.from())).none()
            || self
                .tables
                .aligned(from, to, b.pieces[P::KING].first_piece())
    }
}

impl Default for MoveGenerator {
    fn default() -> Self {
        Self::new()
    }
}
