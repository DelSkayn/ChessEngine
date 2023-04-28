use crate::Move;

/// Trait which a list must implement to be used by the move generator.
pub trait MoveList {
    /// Add a move to the list.
    fn push(&mut self, m: Move);

    /// Remove all moves.
    fn clear(&mut self);

    /// returns the amount of moves.
    fn len(&self) -> usize;
}

pub struct MoveCount(usize);

impl MoveCount {
    pub fn new() -> Self {
        MoveCount(0)
    }
}

impl MoveList for MoveCount {
    fn push(&mut self, _m: Move) {
        self.0 += 1;
    }

    fn clear(&mut self) {
        self.0 = 0;
    }

    fn len(&self) -> usize {
        self.0
    }
}

impl MoveList for () {
    fn push(&mut self, _m: Move) {}

    fn clear(&mut self) {}

    fn len(&self) -> usize {
        0
    }
}

impl MoveList for Vec<Move> {
    fn push(&mut self, m: Move) {
        self.push(m);
    }

    fn clear(&mut self) {
        self.clear()
    }

    fn len(&self) -> usize {
        self.len()
    }
}
