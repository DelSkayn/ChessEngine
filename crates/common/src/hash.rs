use std::{marker::PhantomData, sync::Once};

use crate::{BoardArray, Piece, PieceArray, Square};
use rand::Rng;

static mut PIECES: PieceArray<BoardArray<u64>> = PieceArray::new(BoardArray::new(0));
static mut CASTLE_STATE: [u64; 16] = [0u64; 16];
static mut EN_PASSANT_STATE: [u64; 9] = [0u64; 9];
static mut TURN: u64 = 0;
static mut INITIAL: u64 = 0;

static INIT: Once = Once::new();

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct HashTables(PhantomData<()>);

impl HashTables {
    pub fn new() -> Self {
        INIT.call_once(|| unsafe { Self::init() });
        HashTables(PhantomData)
    }

    unsafe fn init() {
        use rand::SeedableRng;

        let mut rng = rand::rngs::StdRng::seed_from_u64(9291248438346573197);

        for p in Piece::all() {
            for s in 0..64 {
                let s = Square::new(s);

                PIECES[p][s] = rng.gen();
            }
        }
        for x in &mut CASTLE_STATE {
            *x = rng.gen();
        }
        for x in &mut EN_PASSANT_STATE {
            *x = rng.gen();
        }
        TURN = rng.gen();
        INITIAL = rng.gen();
    }

    pub fn pieces(self) -> &'static PieceArray<BoardArray<u64>> {
        unsafe { &PIECES }
    }

    pub fn castle_state(self) -> &'static [u64; 16] {
        unsafe { &CASTLE_STATE }
    }

    pub fn en_passant_state(self) -> &'static [u64; 9] {
        unsafe { &EN_PASSANT_STATE }
    }

    pub fn turn(self) -> u64 {
        unsafe { TURN }
    }

    pub fn initial(self) -> u64 {
        unsafe { INITIAL }
    }
}

impl Default for HashTables {
    fn default() -> Self {
        Self::new()
    }
}
