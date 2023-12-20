use std::{mem::{self,MaybeUninit}, ops};

enum ListValue<N> {
    Free(Option<usize>),
    Used(N),
}

pub struct List<N> {
    values: Vec<ListValue<N>>,
    free: Option<usize>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct NodeId(pub usize);

impl<N> List<N> {
    pub fn new() -> Self {
        List {
            values: Vec::new(),
            free: None,
        }
    }

    pub fn insert(&mut self, node: N) -> NodeId {
        if let Some(id) = self.free {
            let next_free = match self.values[id] {
                ListValue::Free(x) => x,
                _ => unreachable!(),
            };
            self.free = next_free;
            self.values[id] = ListValue::Used(node);
            NodeId(id)
        } else {
            let id = self.values.len();
            self.values.push(ListValue::Used(node));
            NodeId(id)
        }
    }

    pub fn remove(&mut self, id: NodeId) -> N {
        let pref_free = mem::replace(&mut self.free, Some(id.0));

        match mem::replace(&mut self.values[id.0], ListValue::Free(pref_free)) {
            ListValue::Used(x) => x,
            _ => unreachable!(),
        }
    }

    pub fn clear(&mut self){
        self.values.clear();
        self.free = None;
    }
}

impl<N> ops::Index<NodeId> for List<N> {
    type Output = N;
    #[inline(always)]
    fn index(&self, index: NodeId) -> &Self::Output {
        match self.values[index.0]{
            ListValue::Used(ref v) => v,
            ListValue::Free(_) => panic!()
        }
    }
}

impl<N> ops::IndexMut<NodeId> for List<N> {
    #[inline(always)]
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        match self.values[index.0]{
            ListValue::Used(ref mut v) => v,
            ListValue::Free(_) => panic!()
        }
    }
}

pub struct InlineVec<N, const SIZE: usize> {
    moves: [MaybeUninit<N>; SIZE],
    len: u16,
}

impl<N: Copy, const SIZE: usize> InlineVec<N, SIZE> {
    pub fn new() -> Self {
        InlineVec{
            moves: [MaybeUninit::uninit(); SIZE],
            len: 0,
        }
    }

    pub fn iter(&self) -> InlineIter<N, SIZE> {
        InlineIter {
            len: self.len,
            cur: 0,
            v: &self.moves,
        }
    }

    pub fn push(&mut self, m: N) {
        assert!((self.len as usize) < SIZE);
        self.moves[self.len as usize] = MaybeUninit::new(m);
        self.len += 1;
    }

    pub fn get(&self, idx: usize) -> N {
        assert!(idx < self.len as usize);
        unsafe { self.moves[idx].assume_init() }
    }

    pub fn set(&mut self, idx: usize, m: N) {
        assert!(idx < self.len as usize);
        self.moves[idx as usize] = MaybeUninit::new(m);
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }
    pub fn truncate(&mut self, len: usize) {
        assert!(len <= self.len as usize);
        self.len = len as u16;
    }

    fn swap(&mut self, a: usize, b: usize) {
        self.moves.swap(a, b);
    }
}

pub struct InlineIter<'a, N, const SIZE: usize> {
    len: u16,
    cur: u16,
    v: &'a [MaybeUninit<N>; SIZE],
}

impl<'a, N, const SIZE: usize> Iterator for InlineIter<'a, N, SIZE> {
    type Item = &'a N;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == self.cur {
            return None;
        }
        let res = unsafe { &*self.v.get_unchecked(self.cur as usize).as_ptr() };
        self.cur += 1;
        Some(res)
    }
}
