mod fill_7;
mod magic;

use common::{BoardArray, Square, BB};
use std::sync::Once;

static mut KNIGHT_ATTACKS: BoardArray<BB> = BoardArray::new_array([BB::empty(); 64]);
static mut KING_ATTACKS: BoardArray<BB> = BoardArray::new_array([BB::empty(); 64]);

static mut LINES: BoardArray<BoardArray<BB>> =
    BoardArray::new_array([BoardArray::new_array([BB::empty(); 64]); 64]);

static mut BETWEEN: BoardArray<BoardArray<BB>> =
    BoardArray::new_array([BoardArray::new_array([BB::empty(); 64]); 64]);

static TABLE_INITIALIZED: Once = Once::new();

/// A struct representing initialized global tables.
///
/// Move gen uses a bunch of global tables which need to be initialized once and then accessed
/// often and fast. This struct is a zero-sized type which can only be created by first testing and
/// then if not initialized, initializing the global tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tables(std::marker::PhantomData<()>);

impl Tables {
    pub fn new() -> Self {
        // Since the tables always have the same values
        // Writing new values while in use will not lead to problems with race conditions
        // since the values written are the same as the previous.
        // The only thing that we should prevent is reading from unitialized tables.
        TABLE_INITIALIZED.call_once(|| {
            magic::init();
            knight_attacks_init();
            king_attacks_init();
            lines_init();
            between_init();
        });

        Tables(std::marker::PhantomData)
    }

    #[inline(always)]
    pub fn rook_attacks(self, sq: Square, occupied: BB) -> BB {
        magic::rook_attacks(sq, occupied)
    }

    #[inline(always)]
    pub fn bishop_attacks(self, sq: Square, occupied: BB) -> BB {
        magic::bishop_attacks(sq, occupied)
    }

    #[inline(always)]
    pub fn king_attacks(self, sq: Square) -> BB {
        unsafe { KING_ATTACKS[sq] }
    }

    #[inline(always)]
    pub fn knight_attacks(self, sq: Square) -> BB {
        unsafe { KNIGHT_ATTACKS[sq] }
    }

    #[inline(always)]
    pub fn aligned(self, a: Square, b: Square, c: Square) -> bool {
        unsafe { (LINES[a][b] & BB::square(c)).any() }
    }

    #[inline(always)]
    pub fn line(self, from: Square, to: Square) -> BB {
        unsafe { LINES[from][to] }
    }

    #[inline(always)]
    pub fn between(self, from: Square, to: Square) -> BB {
        unsafe { BETWEEN[from][to] }
    }
}

impl Default for Tables {
    fn default() -> Self {
        Self::new()
    }
}

fn knight_attacks_init() {
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
    unsafe {
        KNIGHT_ATTACKS = res;
    }
}

fn king_attacks_init() {
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
    unsafe {
        KING_ATTACKS = res;
    }
}

fn lines_init() {
    let mut res = BoardArray::new(BoardArray::new(BB::empty()));
    for i in 0..64 {
        let i = Square::new(i);
        let (ifile, irank) = i.to_file_rank();
        for j in 0..64 {
            let j = Square::new(j);
            let (jfile, jrank) = j.to_file_rank();

            if i == j {
                continue;
            }

            if ifile == jfile {
                for k in 0..8 {
                    res[i][j] |= BB::square(Square::from_file_rank(ifile, k))
                }
                continue;
            }

            if irank == jrank {
                for k in 0..8 {
                    res[i][j] |= BB::square(Square::from_file_rank(k, irank))
                }
                continue;
            }

            if (irank as i8 - jrank as i8).abs() == (ifile as i8 - jfile as i8).abs() {
                let rank_step = (jrank as i8 - irank as i8).signum();
                let file_step = (jfile as i8 - ifile as i8).signum();

                res[i][j] |= BB::square(i);

                for k in 0..8 {
                    let rank = irank as i8 + rank_step * k;
                    let file = ifile as i8 + file_step * k;

                    if (0..8).contains(&rank) && (0..8).contains(&file) {
                        let sq = Square::from_file_rank(file as u8, rank as u8);
                        res[i][j] |= BB::square(sq);
                    }

                    let rank = irank as i8 - rank_step * k;
                    let file = ifile as i8 - file_step * k;

                    if (0..8).contains(&rank) && (0..8).contains(&file) {
                        let sq = Square::from_file_rank(file as u8, rank as u8);
                        res[i][j] |= BB::square(sq);
                    }
                }
            }
        }
    }
    unsafe { LINES = res };
}

fn between_init() {
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
                let len = (rank_first as i8 - rank_sec as i8).unsigned_abs();
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

    unsafe { BETWEEN = res };
}
