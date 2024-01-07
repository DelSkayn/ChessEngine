use std::cell::Cell;

use common::Move;

#[repr(align(8))]
#[derive(Clone, Copy, Debug)]
pub struct HashData {
    pub score: i16,
    pub m: Move,
    pub depth_bound: DepthBound,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct HashDepth(u32);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct DepthBound(u32);

impl DepthBound {
    pub const MAX_DEPTH: u32 = u32::MAX >> 2;

    pub const fn new(bound: Bound, depth: HashDepth) -> Self {
        assert!(depth.0 < Self::MAX_DEPTH);
        Self((bound as u8 as u32) << 30 | depth.0)
    }

    pub fn bound(self) -> Bound {
        unsafe { std::mem::transmute((self.0 >> 30) as u8) }
    }

    pub fn depth(self) -> HashDepth {
        HashDepth(self.0 & Self::MAX_DEPTH)
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Bound {
    Exact = 0b00,
    Lower = 0b01,
    Upper = 0b10,
}

#[repr(align(16))]
#[derive(Clone, Copy, Debug)]
pub struct HashEntryPart {
    pub hash: u64,
    pub data: HashData,
}

#[derive(Clone, Copy, Debug)]
pub struct HashEntry {
    depth: HashEntryPart,
    replace: HashEntryPart,
}

impl HashEntry {
    const fn invalid() -> Self {
        let invalid = HashEntryPart {
            hash: 0,
            data: HashData {
                score: 0,
                m: Move::INVALID,
                depth_bound: DepthBound::new(Bound::Exact, HashDepth(0)),
            },
        };
        HashEntry {
            depth: invalid,
            replace: invalid,
        }
    }
}

pub struct HashTable {
    table: Box<[HashEntry]>,
    depth_offset: u32,
    // number of entries filled,
    entries: usize,
    hits: Cell<usize>,
}

impl HashTable {
    pub fn new_size(size: usize) -> Self {
        let len = size / std::mem::size_of::<HashEntry>();
        let table = vec![HashEntry::invalid(); len].into_boxed_slice();
        Self {
            table,
            entries: 0,
            hits: Cell::new(0),
            depth_offset: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }

    pub fn reset_hits(&mut self) {
        self.hits.set(0);
    }

    pub fn hits(&self) -> usize {
        self.hits.get()
    }

    pub fn entries(&self) -> usize {
        self.entries
    }

    pub fn advance_move(&mut self) {
        self.hits = Cell::new(0);
        self.depth_offset += 1;
    }

    pub fn reset(&mut self) {
        self.depth_offset = 0;
        self.entries = 0;
        self.hits = Cell::new(0);
        self.table.iter_mut().for_each(|x| {
            x.depth.hash = 0;
            x.replace.hash = 0;
            x.depth.data.m = Move::INVALID;
            x.replace.data.m = Move::INVALID;
        });
    }

    pub fn hash_depth(&self, depth: u8) -> HashDepth {
        HashDepth(self.depth_offset + depth as u32)
    }

    pub fn lookup(&self, hash: u64) -> Option<&HashData> {
        let entry = &self.table[(hash % self.table.len() as u64) as usize];
        if entry.depth.hash == hash && entry.depth.data.m != Move::INVALID {
            self.hits.set(self.hits.get() + 1);
            return Some(&entry.depth.data);
        }
        if entry.replace.hash == hash && entry.replace.data.m != Move::INVALID {
            self.hits.set(self.hits.get() + 1);
            return Some(&entry.replace.data);
        }
        None
    }

    pub fn store(&mut self, hash: u64, score: i32, m: Move, depth: u8, bound: Bound) {
        let depth = self.hash_depth(depth);
        let score = score.clamp(i16::MIN as i32, i16::MAX as i32) as i16;

        let entry = &mut self.table[(hash % self.table.len() as u64) as usize];

        let is_invalid = entry.depth.data.m == Move::INVALID;
        let should_replace = is_invalid || entry.depth.data.depth_bound.depth() <= depth;

        if should_replace {
            self.entries += is_invalid as usize;
            entry.depth = HashEntryPart {
                hash,
                data: HashData {
                    score,
                    m,
                    depth_bound: DepthBound::new(bound, depth),
                },
            };
            return;
        }
        self.entries += (entry.replace.data.m == Move::INVALID) as usize;
        entry.replace = HashEntryPart {
            hash,
            data: HashData {
                score,
                m,
                depth_bound: DepthBound::new(bound, depth),
            },
        }
    }
}
