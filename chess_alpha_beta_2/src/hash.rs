use chess_core::Move;

#[derive(Debug, Clone, Copy)]
pub enum TableScore {
    Upper(i32),
    Lower(i32),
    Exact(i32),
}

#[derive(Debug, Clone, Copy)]
pub struct TableValue {
    pub hash: u64,
    pub depth: u8,
    pub r#move: Move,
    pub score: TableScore,
}

pub struct HashTable {
    values: Box<[TableValue]>,
    bitmap: u64,
}

impl HashTable {
    pub fn new(size: usize) -> Self {
        let size = size.next_power_of_two() >> 1;
        let bitmap = size as u64 - 1;
        let values = vec![
            TableValue {
                hash: 0,
                depth: 0,
                r#move: Move::INVALID,
                score: TableScore::Upper(i32::MAX)
            };
            size
        ];

        HashTable {
            values: values.into_boxed_slice(),
            bitmap,
        }
    }

    #[inline]
    pub fn get(&self, hash: u64) -> Option<&TableValue> {
        let v = &self.values[(self.bitmap & hash) as usize];
        if v.hash == hash {
            return Some(v);
        }
        None
    }

    #[inline]
    pub fn set(&mut self, v: TableValue) {
        self.values[(self.bitmap & v.hash) as usize] = v;
    }
}
