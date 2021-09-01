pub use super::{EndChain, MoveChain};
use crate::{hash::Hasher, util::PieceArray, ExtraState, Piece, Square, BB};

#[derive(Debug, Clone)]
pub struct HashChain<C: MoveChain = EndChain> {
    pub hash: u64,
    hasher: Hasher,
    next: C,
}

impl HashChain<EndChain> {
    pub fn new() -> Self {
        Self::with(EndChain)
    }
}

impl<C: MoveChain> HashChain<C> {
    pub fn with(chain: C) -> Self {
        HashChain {
            hash: 0,
            hasher: Hasher::new(),
            next: chain,
        }
    }
}

impl<C: MoveChain> MoveChain for HashChain<C> {
    type Next = C;

    #[inline(always)]
    fn next_chain(&self) -> &Self::Next {
        &self.next
    }

    #[inline(always)]
    fn next_chain_mut(&mut self) -> &mut Self::Next {
        &mut self.next
    }

    fn position(&mut self, pieces: &PieceArray<BB>, state: ExtraState) {
        self.hash = self.hasher.build(pieces, state);
    }

    fn move_start(&mut self, state: ExtraState) {
        self.hash ^= self.hasher.black();
        self.hash ^= self.hasher.castle()[state.castle as usize];
    }

    /// Called when a move starts
    fn undo_move_start(&mut self, state: ExtraState) {
        self.hash ^= self.hasher.black();
        self.hash ^= self.hasher.castle()[state.castle as usize];
    }

    fn move_piece(&mut self, piece: Piece, from: Square, to: Square) {
        let hash_array = &self.hasher.pieces()[piece];
        self.hash ^= hash_array[from] ^ hash_array[to];
    }

    fn take_piece(&mut self, taken: Piece, square: Square) {
        self.hash ^= self.hasher.pieces()[taken][square];
    }

    #[inline(always)]
    fn untake_piece(&mut self, taken: Piece, square: Square) {
        self.take_piece(taken, square)
    }

    fn promote_piece(&mut self, piece: Piece, promote: Piece, from: Square, to: Square) {
        self.hash ^= self.hasher.pieces()[piece][from];
        self.hash ^= self.hasher.pieces()[promote][to];
    }

    #[inline(always)]
    fn unpromote_piece(&mut self, piece: Piece, promote: Piece, from: Square, to: Square) {
        self.promote_piece(piece, promote, from, to);
    }

    fn move_end(&mut self, _state: ExtraState) {}
}
