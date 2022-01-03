mod hash;
use crate::{util::PieceArray, ExtraState, Piece, Square, BB};
pub use hash::HashChain;

pub trait MoveChain {
    type Next: MoveChain;

    fn next_chain(&self) -> &Self::Next;
    fn next_chain_mut(&mut self) -> &mut Self::Next;

    /// Setup chain from a new position
    fn position(&mut self, pieces: &PieceArray<BB>, state: ExtraState);

    /// Called when a move starts
    fn move_start(&mut self, state: ExtraState);
    fn move_end(&mut self, state: ExtraState);

    /// Called when a move starts
    fn undo_move_start(&mut self, state: ExtraState);

    fn move_piece(&mut self, piece: Piece, from: Square, to: Square);

    fn take_piece(&mut self, taken: Piece, square: Square);
    fn untake_piece(&mut self, taken: Piece, square: Square);

    fn promote_piece(&mut self, piece: Piece, promote: Piece, from: Square, to: Square);
    fn unpromote_piece(&mut self, piece: Piece, promote: Piece, from: Square, to: Square);
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct EndChain;

impl MoveChain for EndChain {
    type Next = EndChain;

    #[inline(always)]
    fn next_chain(&self) -> &Self::Next {
        self
    }

    #[inline(always)]
    fn next_chain_mut(&mut self) -> &mut Self::Next {
        self
    }

    #[inline(always)]
    fn position(&mut self, _pieces: &PieceArray<BB>, _state: ExtraState) {}

    /// Called when a move starts
    #[inline(always)]
    fn move_start(&mut self, _state: ExtraState) {}
    #[inline(always)]
    fn move_end(&mut self, _state: ExtraState) {}

    /// Called when a move starts
    #[inline(always)]
    fn undo_move_start(&mut self, _state: ExtraState) {}

    #[inline(always)]
    fn move_piece(&mut self, _piece: Piece, _from: Square, _to: Square) {}

    #[inline(always)]
    fn take_piece(&mut self, _taken: Piece, _square: Square) {}
    #[inline(always)]
    fn untake_piece(&mut self, _taken: Piece, _square: Square) {}

    fn promote_piece(&mut self, _piece: Piece, _promote: Piece, _from: Square, _to: Square) {}

    fn unpromote_piece(&mut self, _piece: Piece, _promote: Piece, _from: Square, _to: Square) {}
}
