use std::{cell::UnsafeCell, marker::PhantomData};

use chess_core::Move;

#[derive(Debug, Clone, Copy)]
pub enum TableScore {
    Upper(i32),
    Lower(i32),
    Exact(i32),
}

impl TableScore {
    fn is_exact(&self) -> bool {
        match *self {
            TableScore::Exact(_) => true,
            _ => false,
        }
    }
}

pub enum TableEntry<'a> {
    Hit(HitEntry<'a>),
    Miss(Entry),
}

pub struct HitEntry<'a> {
    entry: Entry,
    marker: PhantomData<&'a HashTable>,
}

impl<'a> HitEntry<'a> {
    #[inline(always)]
    fn get(&self) -> &TableData {
        unsafe { &(*self.entry.0) }
    }

    pub fn score(&self) -> TableScore {
        match self.get().gen_bound & HashTable::BOUND_MASK {
            HashTable::BOUND_UPPER => TableScore::Upper(self.get().score as i32),
            HashTable::BOUND_LOWER => TableScore::Lower(self.get().score as i32),
            HashTable::BOUND_EXACT => TableScore::Exact(self.get().score as i32),
            _ => unreachable!(),
        }
    }

    pub fn r#move(&self) -> Move {
        self.get().r#move
    }

    pub fn depth(&self) -> u8 {
        self.get().depth
    }

    pub fn into_entry(self) -> Entry {
        self.entry
    }
}

pub struct Entry(*mut TableData);

#[derive(Clone, Copy)]
pub struct TableData {
    pub hash: u16,
    pub depth: u8,
    pub r#move: Move,
    pub score: i16,
    pub gen_bound: u8,
}

pub struct HashTable {
    values: UnsafeCell<Box<[Bucket]>>,
    bitmap: u64,
    generation: u16,
}

#[derive(Clone, Copy)]
pub struct Bucket {
    values: [TableData; 4],
}

impl HashTable {
    const CACHE_LINE_SIZE: usize = 32;

    const BOUND_MASK: u8 = 0b11;
    const BOUND_UPPER: u8 = 0b01;
    const BOUND_LOWER: u8 = 0b10;
    const BOUND_EXACT: u8 = 0b11;

    const GEN_CYCLE: u16 = 0xFF + (1 << 2);
    const GEN_MASK: u16 = 0xFF << 2;
    const GEN_INCREMENT: u16 = 1 << 2;

    /// Create a new hash table with the specified size in KB.
    pub fn new(size: usize) -> Self {
        assert_eq!(std::mem::size_of::<TableData>(), 8);
        assert_eq!(std::mem::size_of::<Bucket>(), Self::CACHE_LINE_SIZE);

        let size = size * 1024 / std::mem::size_of::<Bucket>();
        let amount = size.next_power_of_two() >> 1;
        println!("amount: {:b}", amount);
        let bitmap = amount as u64 - 1;
        println!("bitmap: {:b}", bitmap);

        let values = vec![
            Bucket {
                values: [TableData {
                    hash: 0,
                    depth: 0,
                    r#move: Move::INVALID,
                    score: 0,
                    gen_bound: 0,
                }; 4],
            };
            amount
        ];

        HashTable {
            values: UnsafeCell::new(values.into_boxed_slice()),
            bitmap,
            generation: Self::GEN_INCREMENT,
        }
    }

    pub fn increment_generation(&mut self) {
        self.generation = (self.generation + Self::GEN_INCREMENT) % 0xFF;
    }

    pub fn write(&mut self, entry: Entry, hash: u64, depth: u8, score: TableScore, r#move: Move) {
        let entry = unsafe { &mut (*entry.0) };
        let key = hash as u16;

        if entry.hash != key || score.is_exact() || depth > entry.depth {
            let (score, bound) = match score {
                TableScore::Exact(x) => (x, HashTable::BOUND_EXACT),
                TableScore::Lower(x) => (x, HashTable::BOUND_LOWER),
                TableScore::Upper(x) => (x, HashTable::BOUND_UPPER),
            };

            entry.hash = key;
            entry.r#move = r#move;
            entry.gen_bound = (self.generation & Self::GEN_MASK) as u8 | bound;
            entry.score = score as i16;
        }
    }

    unsafe fn get_unsafe<'a>(&'a self, hash: u64) -> TableEntry<'a> {
        let bucket = (*self.values.get()).get_unchecked_mut((hash & self.bitmap) as usize);

        let key = hash as u16;

        for i in 0..6 {
            let v = bucket.values.get_unchecked_mut(i);
            if v.hash == key && v.r#move != Move::INVALID {
                return TableEntry::Hit(HitEntry {
                    entry: Entry(v),
                    marker: PhantomData,
                });
            }
        }

        let replace = bucket
            .values
            .iter_mut()
            .min_by_key(|replace| {
                replace.depth as i16
                    - (Self::GEN_CYCLE
                        .wrapping_add(self.generation)
                        .wrapping_sub(replace.gen_bound as u16)
                        & Self::GEN_MASK) as i16
            })
            .unwrap();
        return TableEntry::Miss(Entry(replace));
    }

    #[inline]
    pub fn get<'a>(&'a self, hash: u64) -> TableEntry<'a> {
        unsafe { self.get_unsafe(hash) }
    }
}
