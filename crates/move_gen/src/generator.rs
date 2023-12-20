//! Move generation routines.

use crate::{
    types::{gen_type, Black, GenType, Player, White},
    InlineBuffer, Tables,
};
use common::{board::Board, ExtraState, Move, Piece, Promotion, Square, SquareContent, BB};
use std::marker::PhantomData;

/// Info about a position used in various move generation functions.
#[derive(Debug)]
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
    pub fn about<P: Player>(table: Tables, b: &Board) -> Self {
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
    pub fn gen_moves<T: GenType>(&self, b: &Board, list: &mut InlineBuffer) -> PositionInfo {
        match b.state.player {
            common::Player::White => {
                TypedMoveGenerator::<White, T>::new(self.tables).gen_moves(b, list)
            }
            common::Player::Black => {
                TypedMoveGenerator::<Black, T>::new(self.tables).gen_moves(b, list)
            }
        }
    }

    #[inline]
    pub fn gen_moves_info<T: GenType>(
        &self,
        b: &Board,
        info: &PositionInfo,
        list: &mut InlineBuffer,
    ) {
        match b.state.player {
            common::Player::White => {
                TypedMoveGenerator::<White, T>::new(self.tables).gen_moves_info(b, info, list)
            }
            common::Player::Black => {
                TypedMoveGenerator::<Black, T>::new(self.tables).gen_moves_info(b, info, list)
            }
        }
    }

    #[inline]
    pub fn gen_info(&self, board: &Board) -> PositionInfo {
        match board.state.player {
            common::Player::White => PositionInfo::about::<White>(self.tables, board),
            common::Player::Black => PositionInfo::about::<Black>(self.tables, board),
        }
    }

    #[inline]
    pub fn check_mate(&self, b: &Board, info: &PositionInfo) -> bool {
        match b.state.player {
            common::Player::White => {
                TypedMoveGenerator::<White, gen_type::All>::new(self.tables).is_checkmate(b, info)
            }
            common::Player::Black => {
                TypedMoveGenerator::<Black, gen_type::All>::new(self.tables).is_checkmate(b, info)
            }
        }
    }

    #[inline]
    pub fn checked_king(&self, b: &Board, info: &PositionInfo) -> bool {
        match b.state.player {
            common::Player::White => TypedMoveGenerator::<White, gen_type::All>::new(self.tables)
                .is_king_checked(b, info),
            common::Player::Black => TypedMoveGenerator::<Black, gen_type::All>::new(self.tables)
                .is_king_checked(b, info),
        }
    }

    pub fn is_legal(&self, m: Move, b: &Board, info: &PositionInfo) -> bool {
        match b.state.player {
            common::Player::White => {
                TypedMoveGenerator::<White, gen_type::All>::new(self.tables).is_legal(m, b, info)
            }
            common::Player::Black => {
                TypedMoveGenerator::<Black, gen_type::All>::new(self.tables).is_legal(m, b, info)
            }
        }
    }

    /// Returns if the game is drawn by material or by move clock.
    pub fn drawn_by_rule(&self, b: &Board, info: &PositionInfo) -> bool {
        if b.state.move_clock >= 50 {
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
}

impl Default for MoveGenerator {
    fn default() -> Self {
        Self::new()
    }
}

struct TypedMoveGenerator<P, T> {
    tables: Tables,
    marker: PhantomData<(*const P, *const T)>,
}

impl<P: Player, T: GenType> TypedMoveGenerator<P, T> {
    pub const fn new(tables: Tables) -> Self {
        Self {
            tables,
            marker: PhantomData,
        }
    }

    pub fn is_king_checked(&self, b: &Board, info: &PositionInfo) -> bool {
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

    pub fn gen_moves(&self, b: &Board, list: &mut InlineBuffer) -> PositionInfo {
        let info = PositionInfo::about::<P>(self.tables, b);
        self.gen_moves_info(b, &info, list);
        panic!();
        info
    }

    #[inline]
    pub fn gen_moves_info(&self, b: &Board, info: &PositionInfo, list: &mut InlineBuffer) {
        let target = if T::QUIET { !info.my } else { info.their };

        if (info.attacked & b.pieces[P::KING]).any() {
            self.gen_evasion(b, info, list, target);
        } else {
            self.gen_moves_pseudo(b, info, list, target);
        }

        /*
        if T::LEGAL {
            let mut cur = 0;
            for i in 0..list.len() {
                let m = list.get(i);
                if self.is_legal_player::<P, _>(m, b, &info) {
                    list.set(cur, m);
                    cur += 1;
                }
            }
            list.truncate(cur);
        }
        */
    }

    pub fn gen_evasion(&self, b: &Board, info: &PositionInfo, list: &mut InlineBuffer, target: BB) {
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
            let m = Move::normal(king_sq, p);
            list.push(m)
        }

        if attackers_count > 1 {
            return;
        }

        let mut blockers = attackers;
        if T::QUIET && (attackers & b.pieces[P::Opponent::KNIGHT]).none() {
            blockers |= self.tables.between(attackers.first_piece(), king_sq);
        }
        self.gen_moves_sliders(b, info, list, blockers);
        self.gen_moves_knight(b, info, list, blockers);
        self.gen_pawn_moves(b, info, list, blockers);
    }

    pub fn gen_moves_pseudo(
        &self,
        b: &Board,
        info: &PositionInfo,
        list: &mut InlineBuffer,
        target: BB,
    ) {
        let king_sq = b.pieces[P::KING].first_piece();
        for s in self.tables.king_attacks(king_sq) & target & !info.attacked {
            let m = Move::normal(king_sq, s);
            list.push(m)
        }

        let mut pawn_target = target;
        if T::CHECKS {
            pawn_target |= b.pieces[P::Opponent::KING].shift(P::Opponent::ATTACK_LEFT)
                | b.pieces[P::Opponent::KING].shift(P::Opponent::ATTACK_LEFT)
        }
        self.gen_pawn_moves(b, info, list, pawn_target);

        let mut knight_target = target;
        if T::CHECKS {
            knight_target |= self
                .tables
                .knight_attacks(b.pieces[P::Opponent::KING].first_piece())
        }
        self.gen_moves_knight(b, info, list, knight_target);

        self.gen_moves_sliders(b, info, list, target);

        if T::QUIET {
            self.gen_castle(b, info, list);
        }
    }

    pub fn gen_pawn_moves(
        &self,
        b: &Board,
        info: &PositionInfo,
        list: &mut InlineBuffer,
        target: BB,
    ) {
        let tgt = target & !P::RANK_8;
        let tgt_promote = target & P::RANK_8;

        let attacked = b.pieces[P::PAWN].shift(P::ATTACK_LEFT) & info.their;
        for p in attacked & tgt {
            let m = Move::normal(p - P::ATTACK_LEFT.as_offset(), p);
            if self.is_legal(m, b, info) {
                list.push(m);
            }
        }
        for p in attacked & tgt_promote {
            let from = p - P::ATTACK_LEFT.as_offset();
            let to = p;
            if self.is_legal(Move::normal(from, to), b, info) {
                self.gen_promotions(from, to, list);
            }
        }

        let attacked = b.pieces[P::PAWN].shift(P::ATTACK_RIGHT) & info.their;
        for p in attacked & tgt {
            let m = Move::normal(p - P::ATTACK_RIGHT.as_offset(), p);
            if self.is_legal(m, b, info) {
                list.push(m);
            }
        }

        for p in attacked & tgt_promote {
            let from = p - P::ATTACK_RIGHT.as_offset();
            let to = p;
            if self.is_legal(Move::normal(from, to), b, info) {
                self.gen_promotions(p - P::ATTACK_RIGHT.as_offset(), p, list);
            }
        }

        let moved = b.pieces[P::PAWN].shift(P::PAWN_MOVE) & !info.occupied;
        let pawn_move = P::PAWN_MOVE.as_offset();
        for p in moved & tgt {
            let m = Move::normal(p - pawn_move, p);
            if self.is_legal(m, b, info) {
                list.push(m);
            }
        }

        for p in moved & tgt_promote {
            let from = p - pawn_move;
            let to = p;
            if self.is_legal(Move::normal(from, to), b, info) {
                self.gen_promotions(from, to, list);
            }
        }

        let double_moved = (moved & P::RANK_3).shift(P::PAWN_MOVE) & !info.occupied;
        let double_pawn_move = pawn_move + pawn_move;
        for p in double_moved & tgt {
            let m = Move::double_pawn(p - double_pawn_move, p);
            if self.is_legal(m, b, info) {
                list.push(m);
            }
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
                let m = Move::en_passant(pawn.first_piece(), sq + P::ATTACK_RIGHT.as_offset());
                if self.is_legal(m, b, info) {
                    list.push(m);
                }
            }
            let pawn = file.shift(P::RIGHT) & pawn_on_rank;
            if pawn.any() {
                let sq = pawn.first_piece();
                let m = Move::en_passant(pawn.first_piece(), sq + P::ATTACK_LEFT.as_offset());
                if self.is_legal(m, b, info) {
                    list.push(m);
                }
            }
        }
    }

    pub fn gen_castle(&self, b: &Board, info: &PositionInfo, list: &mut InlineBuffer) {
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
                // Should always be legal given tasts.
                list.push(Move::castle(from, to));
            }
        }
        if b.state.castle & queen_castle != 0 {
            let empty = occupied & (CASTLE_QUEEN_EMPTY_MASK << P::CASTLE_SHIFT);
            let attacked = info.attacked & (CASTLE_QUEEN_ATTACKED_MASK << P::CASTLE_SHIFT);
            if !((empty | attacked).any()) {
                let to = P::CASTLE_QUEEN_TO;
                // Should always be legal given tasts.
                list.push(Move::castle(from, to));
            }
        }
    }

    /// Generate promotions
    /// Caller should check if the promotions are legal
    pub fn gen_promotions(&self, from: Square, to: Square, list: &mut InlineBuffer) {
        list.push(Move::promotion(from, to, Promotion::Queen));
        list.push(Move::promotion(from, to, Promotion::Rook));
        list.push(Move::promotion(from, to, Promotion::Knight));
        list.push(Move::promotion(from, to, Promotion::Bishop));
    }

    pub fn gen_moves_sliders(
        &self,
        b: &Board,
        info: &PositionInfo,
        list: &mut InlineBuffer,
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
            for bish in bishops {
                for m in self.tables.bishop_attacks(bish, info.occupied) & target {
                    let m = Move::normal(bish, m);
                    if self.is_legal(m, b, info) {
                        list.push(m);
                    }
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
            for r in rooks {
                for m in self.tables.rook_attacks(r, info.occupied) & target {
                    let m = Move::normal(r, m);
                    if self.is_legal(m, b, info) {
                        list.push(m);
                    }
                }
            }
        }
    }

    pub fn gen_moves_knight(
        &self,
        b: &Board,
        info: &PositionInfo,
        list: &mut InlineBuffer,
        target: BB,
    ) {
        for k in b.pieces[P::KNIGHT] {
            // If the knight is pinned it can never be moved.
            // Otherwise all its pseudo legal moves are legal.
            if T::LEGAL && (info.blockers & BB::square(k)).any() {
                continue;
            }
            for a in self.tables.knight_attacks(k) & target {
                list.push(Move::normal(k, a));
            }
        }
    }
    pub fn is_legal(&self, m: Move, b: &Board, info: &PositionInfo) -> bool {
        if !T::LEGAL {
            return true;
        }

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

        if SquareContent::from(P::KING) == b.on(from) {
            // The generator does not generate pseudo legal king moves.
            return true;
        }

        (info.blockers & BB::square(m.from())).none()
            || self
                .tables
                .aligned(from, to, b.pieces[P::KING].first_piece())
    }
}

impl<P: Player, T: GenType> TypedMoveGenerator<P, T> {
    pub fn is_checkmate(&self, b: &Board, info: &PositionInfo) -> bool {
        let king_sq = b.pieces[P::KING].first_piece();

        // which pieces attack the king
        let attackers = self.tables.bishop_attacks(king_sq, info.occupied)
            & (b.pieces[P::Opponent::QUEEN] | b.pieces[P::Opponent::BISHOP])
            | self.tables.rook_attacks(king_sq, info.occupied)
                & (b.pieces[P::Opponent::QUEEN] | b.pieces[P::Opponent::ROOK])
            | self.tables.knight_attacks(king_sq) & b.pieces[P::Opponent::KNIGHT]
            | b.pieces[P::KING].shift(P::ATTACK_LEFT) & b.pieces[P::Opponent::PAWN]
            | b.pieces[P::KING].shift(P::ATTACK_RIGHT) & b.pieces[P::Opponent::PAWN];

        let attackers_count = attackers.count();

        // no attackers, can't mate.
        if attackers_count == 0 {
            return false;
        }

        // Does the king have any non attacked squares it can move to.
        let mut king_moves = self.tables.king_attacks(king_sq) & !(info.my | info.attacked);
        for p in attackers & !(b.pieces[P::Opponent::KNIGHT] | b.pieces[P::Opponent::PAWN]) {
            king_moves &= !(self.tables.line(king_sq, p) & !BB::square(p));
        }

        if king_moves.any() {
            return false;
        }

        // if there are multiple attackers they can't be blocked or taken.
        if attackers_count > 1 {
            return true;
        }

        let blocking_squares = if (attackers & b.pieces[P::Opponent::KNIGHT]).any() {
            BB::EMPTY
        } else {
            self.tables.between(attackers.first_piece(), king_sq)
        };
        let blockers = attackers | blocking_squares;

        // can a bishop-like block or take the attack
        for bish in b.pieces[P::BISHOP] | b.pieces[P::QUEEN] {
            if (self.tables.bishop_attacks(bish, info.occupied) & blockers).any() {
                return false;
            }
        }

        // can a rook-like block or take the attack
        for rook in b.pieces[P::ROOK] | b.pieces[P::QUEEN] {
            if (self.tables.rook_attacks(rook, info.occupied) & blockers).any() {
                return false;
            }
        }

        // can a knight block or take the attack
        for knight in b.pieces[P::KNIGHT] {
            if (self.tables.knight_attacks(knight) & blockers).any() {
                return false;
            }
        }

        // can a pawn take the attacker
        if ((b.pieces[P::PAWN].shift(P::ATTACK_LEFT) | b.pieces[P::PAWN].shift(P::ATTACK_RIGHT))
            & attackers)
            .any()
        {
            return false;
        }

        if blocking_squares.none() {
            return true;
        }

        // can a pawn move inbetween the attacker
        let moved = b.pieces[P::PAWN].shift(P::PAWN_MOVE);
        if (moved & blocking_squares).any() {
            return false;
        }

        // can a pawn double move in fron of the attacker.
        if ((moved & !info.occupied & P::RANK_3).shift(P::PAWN_MOVE) & blocking_squares).any() {
            return false;
        };

        true
    }
}
