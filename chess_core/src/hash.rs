//! Utilities for hashing a board state.

use crate::{
    util::{BoardArray, PieceArray},
    ExtraState, Piece, Player, Square, BB,
};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::sync::Once;

static HASH_INITIALIZED: Once = Once::new();

static mut PIECES: PieceArray<BoardArray<u64>> =
    PieceArray::new_array([BoardArray::new_array([0; 64]); 12]);
static mut CASTLE: [u64; 16] = [0; 16];
static mut BLACK: u64 = 0;

/// A zobrist hashing table for a position
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct Hasher;

impl Hasher {
    pub fn new() -> Self {
        HASH_INITIALIZED.call_once(|| {
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

            unsafe {
                PIECES = pieces;
                CASTLE = castle;
                BLACK = black;
            }
        });

        Hasher
    }

    #[inline(always)]
    pub fn pieces(&self) -> &PieceArray<BoardArray<u64>> {
        unsafe { &PIECES }
    }

    #[inline(always)]
    pub fn castle(&self) -> &[u64; 16] {
        unsafe { &CASTLE }
    }

    #[inline(always)]
    pub fn black(&self) -> u64 {
        unsafe { BLACK }
    }

    pub fn build(&self, pieces: &PieceArray<BB>, state: ExtraState) -> u64 {
        let mut res = 0;

        for p in Piece::WhiteKing.to(Piece::BlackPawn) {
            for s in pieces[p].iter() {
                res ^= self.pieces()[p][s]
            }
        }
        res ^= self.castle()[state.castle as usize];
        if let Player::Black = state.player {
            res ^= self.black();
        }

        res
    }
}
