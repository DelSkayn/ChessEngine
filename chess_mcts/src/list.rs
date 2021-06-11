use std::{mem, ops};

enum ListValue<N> {
    Free(Option<usize>),
    Used(N),
}

pub struct List<N> {
    values: Vec<ListValue<N>>,
    free: Option<usize>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct NodeId(usize);

impl List<N> {
    pub fn new() -> Self {
        Tree {
            nodes: Vec::new(),
            free: None,
        }
    }

    pub fn insert(&mut self, node: N) -> NodeId {
        if let Some(id) = self.free {
            let next_free = match self.nodes[id] {
                ListValue::Free(x) => x,
                _ => unreachable!(),
            };
            self.free = next_free;
            self.nodes[id] = ListValue::Used(node);
        } else {
            let id = nodes.len();
            nodes.push(ListValue::Used(node));
            NodeId(id)
        }
    }

    pub fn remove(&mut self, id: NodeId) -> N {
        let pref_free = mem::replace(&mut self.free, Some(id.0));

        match mem::replace(&mut self.nodes[id], ListValue::Free(pref_free)) {
            ListValue::Used(x) => x,
            _ => unreachable!(),
        }
    }
}

impl<N> ops::Index<NodeId> for List<N> {
    type Output = N;
    #[inline(always)]
    pub fn index(&self, index: NodeId) -> &Self::Output {
        &self.nodes[index.0]
    }
}

impl<N> ops::IndexMut<NodeId> for List<N> {
    #[inline(always)]
    pub fn index_mut(&mut self, index: NodeId) -> &Self::Output {
        &mut self.nodes[index.0]
    }
}

pub struct InlineList<N, const SIZE: usize> {
    moves: [MaybeUninit<N>; SIZE],
    len: u16,
}

impl<N: Copy, const SIZE: usize> InlineList<N, SIZE> {
    pub fn new() -> Self {
        InlineBuffer {
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

    fn push(&mut self, m: N) {
        assert!((self.len as usize) < SIZE);
        self.moves[self.len as usize] = MaybeUninit::new(m);
        self.len += 1;
    }

    fn get(&self, idx: usize) -> N {
        assert!(idx < self.len as usize);
        unsafe { self.moves[idx].assume_init() }
    }

    fn set(&mut self, idx: usize, m: N) {
        assert!(idx < self.len as usize);
        self.moves[idx as usize] = MaybeUninit::new(m);
    }

    fn clear(&mut self) {
        self.len = 0;
    }

    fn len(&self) -> usize {
        self.len as usize
    }
    fn truncate(&mut self, len: usize) {
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
    v: &'a [MaybeUninit<Move>; SIZE],
}

impl<'a, N, const SIZE: usize> Iterator for InlineIter<'a, N, SIZE> {
    type Item = &'a Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == self.cur {
            return None;
        }
        let res = unsafe { &*self.v.get_unchecked(self.cur as usize).as_ptr() };
        self.cur += 1;
        Some(res)
    }
}
