use chess_core::{gen::fill_7, Square, BB};
use rand::Rng;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Mutex,
};

fn rook_sliding_attack(s: Square, occupied: BB) -> BB {
    let s = BB::square(s);
    fill_7::n(s, !occupied)
        | fill_7::w(s, !occupied)
        | fill_7::s(s, !occupied)
        | fill_7::e(s, !occupied)
}

fn bishop_sliding_attack(s: Square, occupied: BB) -> BB {
    let s = BB::square(s);
    fill_7::nw(s, !occupied)
        | fill_7::ne(s, !occupied)
        | fill_7::sw(s, !occupied)
        | fill_7::se(s, !occupied)
}

fn edges(s: Square) -> BB {
    (BB::RANK_1 | BB::RANK_8) & !(BB::RANK_1 << s.rank() * 8)
        | (BB::FILE_A | BB::FILE_H) & !(BB::FILE_A << s.file())
}

fn rook_mask(s: Square) -> BB {
    rook_sliding_attack(s, BB::EMPTY) & !edges(s)
}

fn bishop_mask(s: Square) -> BB {
    bishop_sliding_attack(s, BB::EMPTY) & !edges(s)
}

pub struct Possible {
    occupied: BB,
    attack: BB,
}

fn init_possible(s: Square, rook: bool) -> Vec<Possible> {
    let mask = if rook { rook_mask(s) } else { bishop_mask(s) };

    let bits: Vec<Square> = mask.iter().collect();
    let mut possible = Vec::with_capacity(1 << bits.len());

    for i in 0..1 << bits.len() {
        let mut occ = BB::empty();
        for j in 0..bits.len() {
            if (1 << j) & i != 0 {
                occ |= BB::square(bits[j])
            }
        }
        possible.push(Possible {
            occupied: occ,
            attack: if rook {
                rook_sliding_attack(s, occ)
            } else {
                bishop_sliding_attack(s, occ)
            },
        })
    }
    possible
}

struct ThreadCtx {
    run: AtomicBool,
    best_range: AtomicU64,
    best_magic: Mutex<Option<FoundMagic>>,
    magics_searched: AtomicU64,
}

impl ThreadCtx {
    fn new(shift: u64) -> Self {
        let run = AtomicBool::new(true);
        let best_range = AtomicU64::new(1 << shift);
        let best_magic = Mutex::<Option<FoundMagic>>::new(None);
        let magics_searched = AtomicU64::new(0);
        ThreadCtx {
            run,
            best_range,
            best_magic,
            magics_searched,
        }
    }

    fn search_magic(&self) {
        self.magics_searched.fetch_add(1, Ordering::AcqRel);
    }

    fn best_range(&self) -> u64 {
        self.best_range.load(Ordering::Acquire)
    }

    fn update_magic(&self, found: FoundMagic) {
        if self.best_range.fetch_min(found.range, Ordering::AcqRel) > found.range {
            let mut lock = self.best_magic.lock().unwrap();
            if let Some(magic) = *lock {
                if magic.range > found.range {
                    println!("found better: {:?}", found);
                    *lock = Some(found);
                }
            } else {
                println!("found better: {:?}", found);
                *lock = Some(found);
            }
        }
    }

    fn run(&self) -> bool {
        self.run.load(Ordering::Acquire)
    }

    fn stop(&self) {
        self.run.store(false, Ordering::Release)
    }
}

const BASE_MAGICS: [u64; 64] = [
    0x00280077ffebfffe,
    0x2004010201097fff,
    0x0010020010053fff,
    0x0040040008004002,
    0x7fd00441ffffd003,
    0x4020008887dffffe,
    0x004000888847ffff,
    0x006800fbff75fffd,
    0x000028010113ffff,
    0x0020040201fcffff,
    0x007fe80042ffffe8,
    0x00001800217fffe8,
    0x00001800073fffe8,
    0x00001800e05fffe8,
    0x00001800602fffe8,
    0x000030002fffffa0,
    0x00300018010bffff,
    0x0003000c0085fffb,
    0x0004000802010008,
    0x0004002020020004,
    0x0001002002002001,
    0x0001001000801040,
    0x0000004040008001,
    0x0000006800cdfff4,
    0x0040200010080010,
    0x0000080010040010,
    0x0004010008020008,
    0x0000040020200200,
    0x0002008010100100,
    0x0000008020010020,
    0x0000008020200040,
    0x0000820020004020,
    0x00fffd1800300030,
    0x007fff7fbfd40020,
    0x003fffbd00180018,
    0x001fffde80180018,
    0x000fffe0bfe80018,
    0x0001000080202001,
    0x0003fffbff980180,
    0x0001fffdff9000e0,
    0x00fffefeebffd800,
    0x007ffff7ffc01400,
    0x003fffbfe4ffe800,
    0x001ffff01fc03000,
    0x000fffe7f8bfe800,
    0x0007ffdfdf3ff808,
    0x0003fff85fffa804,
    0x0001fffd75ffa802,
    0x00ffffd7ffebffd8,
    0x007fff75ff7fbfd8,
    0x003fff863fbf7fd8,
    0x001fffbfdfd7ffd8,
    0x000ffff810280028,
    0x0007ffd7f7feffd8,
    0x0003fffc0c480048,
    0x0001ffffafd7ffd8,
    0x00ffffe4ffdfa3ba,
    0x007fffef7ff3d3da,
    0x003fffbfdfeff7fa,
    0x001fffeff7fbfc22,
    0x0000020408001001,
    0x0007fffeffff77fd,
    0x0003ffffbf7dfeec,
    0x0001ffff9dffa333,
];

fn run_thread(sq: Square, rook: bool, shift: u64, ctx: &ThreadCtx) {
    let possible = init_possible(sq, rook);
    let mut used = vec![false; 1 << shift];
    let mut attack = vec![BB::FULL; 1 << shift];

    let mut first = true;

    while ctx.run() {
        used.fill(false);
        attack.fill(BB::FULL);
        ctx.search_magic();
        let magic = if first {
            first = false;
            BASE_MAGICS[sq.get() as usize]
        } else {
            rand::thread_rng().gen()
        };
        let mut low = 1 << shift;
        let mut high = 0;
        let mut found = true;

        let b_range = ctx.best_range();
        for p in possible.iter() {
            let idx = p.occupied.get().wrapping_mul(magic);
            let idx = idx >> (64 - shift);
            if used[idx as usize] && attack[idx as usize] != p.attack {
                found = false;
                break;
            }
            low = low.min(idx);
            high = high.max(idx);
            let range = high - low;
            if range >= b_range {
                found = false;
                break;
            }
            used[idx as usize] = true;
            attack[idx as usize] = p.attack;
        }

        if found {
            let range = high - low;
            ctx.update_magic(FoundMagic { magic, range });
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FoundMagic {
    magic: u64,
    range: u64,
}

fn main() {
    let mut total_size = 0;
    let mut cur_s = 0;
    while cur_s < 64 {
        let sq = Square::new(cur_s);
        println!("Generating magics for {}", sq);
        let shift = 12;

        let ctx = ThreadCtx::new(shift);
        rayon::scope(|s| {
            let num_threads = rayon::current_num_threads();
            for _ in 0..num_threads {
                s.spawn(|_| run_thread(sq, true, shift, &ctx));
            }

            std::thread::sleep(std::time::Duration::from_secs(20));
            ctx.stop();
        });

        println!("{}", ctx.best_range.load(Ordering::Acquire));
        println!(
            "magics_searched: {}",
            ctx.magics_searched.load(Ordering::Acquire)
        );
        let lock = ctx.best_magic.lock().unwrap();
        if let Some(x) = *lock {
            println!("found magic {:x} with size {}", x.magic, x.range);
            total_size += x.range;
            cur_s += 1;
        } else {
            println!("failed to find magic");
            cur_s += 1;
        }
    }
    println!("total table size: {}", total_size);
}
