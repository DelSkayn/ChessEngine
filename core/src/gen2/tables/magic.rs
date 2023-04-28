use crate::gen::fill_7;
use crate::{bb::BB, util::BoardArray, Square};

const BISHOP_MAGIC: BoardArray<u64> = BoardArray::new_array([
    0x007fbfbfbfbfbfff,
    0x0000a060401007fc,
    0x0001004008020000,
    0x0000806004000000,
    0x0000100400000000,
    0x000021c100b20000,
    0x0000040041008000,
    0x00000fb0203fff80,
    0x0000040100401004,
    0x0000020080200802,
    0x0000004010202000,
    0x0000008060040000,
    0x0000004402000000,
    0x0000000801008000,
    0x000007efe0bfff80,
    0x0000000820820020,
    0x0000400080808080,
    0x00021f0100400808,
    0x00018000c06f3fff,
    0x0000258200801000,
    0x0000240080840000,
    0x000018000c03fff8,
    0x00000a5840208020,
    0x0000020008208020,
    0x0000804000810100,
    0x0001011900802008,
    0x0000804000810100,
    0x000100403c0403ff,
    0x00078402a8802000,
    0x0000101000804400,
    0x0000080800104100,
    0x00004004c0082008,
    0x0001010120008020,
    0x000080809a004010,
    0x0007fefe08810010,
    0x0003ff0f833fc080,
    0x007fe08019003042,
    0x003fffefea003000,
    0x0000101010002080,
    0x0000802005080804,
    0x0000808080a80040,
    0x0000104100200040,
    0x0003ffdf7f833fc0,
    0x0000008840450020,
    0x00007ffc80180030,
    0x007fffdd80140028,
    0x00020080200a0004,
    0x0000101010100020,
    0x0007ffdfc1805000,
    0x0003ffefe0c02200,
    0x0000000820806000,
    0x0000000008403000,
    0x0000000100202000,
    0x0000004040802000,
    0x0004010040100400,
    0x00006020601803f4,
    0x0003ffdfdfc28048,
    0x0000000820820020,
    0x0000000008208060,
    0x0000000000808020,
    0x0000000001002020,
    0x0000000401002008,
    0x0000004040404040,
    0x007fff9fdf7ff813,
]);
const BISHOP_OFFSET: BoardArray<u32> = BoardArray::new_array([
    5378, 4093, 4314, 6587, 6491, 6330, 5609, 22236, 6106, 5625, 16785, 16817, 6842, 7003, 4197,
    7356, 4602, 4538, 29531, 45393, 12420, 15763, 5050, 4346, 6074, 7866, 32139, 57673, 55365,
    15818, 5562, 6390, 7930, 13329, 7170, 27267, 53787, 5097, 6643, 6138, 7418, 7898, 42012, 57350,
    22813, 56693, 5818, 7098, 4451, 4709, 4794, 13364, 4570, 4282, 14964, 4026, 4826, 7354, 4848,
    15946, 14932, 16588, 6905, 16076,
]);

const ROOK_MAGIC: BoardArray<u64> = BoardArray::new_array([
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
]);

const ROOK_OFFSET: BoardArray<u32> = BoardArray::new_array([
    26304, 35520, 38592, 8026, 22196, 80870, 76747, 30400, 11115, 18205, 53577, 62724, 34282,
    29196, 23806, 49481, 2410, 36498, 24478, 10074, 79315, 51779, 13586, 19323, 70612, 83652,
    63110, 34496, 84966, 54341, 60421, 86402, 50245, 76622, 84676, 78757, 37346, 370, 42182, 45385,
    61659, 12790, 16762, 0, 38380, 11098, 21803, 39189, 58628, 44116, 78357, 44481, 64134, 41759,
    1394, 40910, 66516, 3897, 3930, 72934, 72662, 56325, 66501, 14826,
]);

static mut ROOK_MASK: BoardArray<BB> = BoardArray::new_array([BB::EMPTY; 64]);
static mut BISHOP_MASK: BoardArray<BB> = BoardArray::new_array([BB::EMPTY; 64]);

static mut MAGIC_TABLE: [BB; 88772] = [BB::empty(); 88772];

pub fn init() {
    let mut bishop_mask = BoardArray::new(BB::empty());
    let mut rook_mask = BoardArray::new(BB::empty());

    for i in 0..64 {
        let s = Square::new(i);
        bishop_mask[s] = gen_bishop_mask(s);
        rook_mask[s] = gen_rook_mask(s);
    }
    unsafe {
        ROOK_MASK = rook_mask;
        BISHOP_MASK = bishop_mask;
    }
    unsafe { init_magic() }
}

unsafe fn init_magic() {
    for s in 0..64 {
        let s = Square::new(s);
        let mut occ = BB::empty();
        loop {
            let idx = occ.get().wrapping_mul(BISHOP_MAGIC[s]);
            let idx = (idx >> 64 - 9) as u32 + BISHOP_OFFSET[s];
            MAGIC_TABLE[idx as usize] = bishop_sliding_attack(s, occ);

            occ = occ.sub(BISHOP_MASK[s]) & BISHOP_MASK[s];
            if occ.none() {
                break;
            }
        }

        let mut occ = BB::empty();
        loop {
            let idx = occ.get().wrapping_mul(ROOK_MAGIC[s]);
            let idx = (idx >> 64 - 12) as u32 + ROOK_OFFSET[s];
            MAGIC_TABLE[idx as usize] = rook_sliding_attack(s, occ);

            occ = occ.sub(ROOK_MASK[s]) & ROOK_MASK[s];
            if occ.none() {
                break;
            }
        }
    }
}

pub fn rook_attacks(s: Square, occ: BB) -> BB {
    unsafe {
        let idx = (occ & ROOK_MASK[s]).get().wrapping_mul(ROOK_MAGIC[s]);
        let idx = (idx >> 64 - 12) as u32 + ROOK_OFFSET[s];
        MAGIC_TABLE[idx as usize]
    }
}

pub fn bishop_attacks(s: Square, occ: BB) -> BB {
    unsafe {
        let idx = (occ & BISHOP_MASK[s]).get().wrapping_mul(BISHOP_MAGIC[s]);
        let idx = (idx >> 64 - 9) as u32 + BISHOP_OFFSET[s];
        MAGIC_TABLE[idx as usize]
    }
}

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

fn gen_rook_mask(s: Square) -> BB {
    rook_sliding_attack(s, BB::EMPTY) & !edges(s)
}

fn gen_bishop_mask(s: Square) -> BB {
    bishop_sliding_attack(s, BB::EMPTY) & !edges(s)
}
