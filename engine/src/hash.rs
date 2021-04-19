use crate::{
    util::{BoardArray, PieceArray},
    ExtraState, Piece, Square, BB,
};
use rand::{rngs::StdRng, Rng, SeedableRng};

/// A zobrist hashing table;
#[derive(Clone)]
pub struct Hasher {
    pub pieces: PieceArray<BoardArray<u64>>,
    pub castle: [u64; 16],
    pub black: u64,
}

impl Hasher {
    pub fn new() -> Self {
        let mut pieces = PieceArray::new(BoardArray::new(0));
        let mut random = StdRng::seed_from_u64(0xDEADBEEFCAFEBABE);
        for p in Piece::WhiteKing.to(Piece::BlackPawn) {
            for s in 0..64 {
                let square = Square::new(s);
                pieces[p][square] = random.gen();
            }
        }
        let mut castle = [0; 16];
        for i in 0..16 {
            castle[i] = random.gen();
        }
        let black = random.gen();

        Hasher {
            pieces,
            castle,
            black,
        }
    }

    pub fn build(&self, pieces: [BB; 12], state: ExtraState) -> u64 {
        let mut res = 0;

        for p in Piece::WhiteKing.to(Piece::BlackPawn) {
            for s in pieces[p as usize].iter() {
                res ^= self.pieces[p][s]
            }
        }
        res ^= self.castle[state.castle as usize];
        if state.black_turn {
            res ^= self.black;
        }

        res
    }
}
