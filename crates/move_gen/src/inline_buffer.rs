use common::Move;
use std::mem::MaybeUninit;

/// The maximum number of moves in any position.
pub const MAX_MOVES: usize = 218;

pub struct InlineBuffer<const SIZE: usize = MAX_MOVES, T = Move> {
    moves: [MaybeUninit<T>; SIZE],
    len: u8,
}

impl<const SIZE: usize, T: Copy> Clone for InlineBuffer<SIZE, T> {
    fn clone(&self) -> Self {
        let mut moves = [MaybeUninit::uninit(); SIZE];
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.moves.as_ptr(),
                moves.as_mut_ptr(),
                self.len as usize,
            )
        };
        Self {
            moves,
            len: self.len,
        }
    }
}

impl<const SIZE: usize, T: Copy> InlineBuffer<SIZE, T> {
    #[inline]
    pub fn new() -> Self {
        debug_assert!(SIZE <= u8::MAX.into());
        InlineBuffer {
            moves: [MaybeUninit::uninit(); SIZE],
            len: 0,
        }
    }

    #[inline]
    pub fn iter(&self) -> InlineIter<SIZE, T> {
        InlineIter {
            len: self.len,
            cur: 0,
            v: &self.moves,
        }
    }

    #[inline]
    pub fn swap_remove(&mut self, idx: u8) {
        assert!(idx < self.len, "got idx: {} while len is {}", idx, self.len);
        self.moves.swap(idx.into(), self.len as usize - 1);
        self.len -= 1;
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.len > 0 {
            self.len -= 1;
            Some(unsafe { self.moves[self.len as usize].assume_init() })
        } else {
            None
        }
    }

    #[inline]
    pub fn len(&self) -> u8 {
        self.len
    }

    #[inline]
    pub fn truncate(&mut self, len: u8) {
        assert!(len <= self.len);
        self.len = len;
    }

    #[inline]
    pub fn swap(&mut self, a: u8, b: u8) {
        self.moves.swap(a.into(), b.into());
    }

    pub fn get(&self, at: u8) -> Option<T> {
        if at >= self.len {
            return None;
        }
        Some(unsafe { self.moves[at as usize].assume_init() })
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn push(&mut self, m: T) {
        assert!((self.len as usize) < SIZE);
        self.moves[self.len as usize] = MaybeUninit::new(m);
        self.len += 1;
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn as_slice(&self) -> &[Move] {
        unsafe { std::slice::from_raw_parts::<Move>(self.moves.as_ptr().cast(), self.len as usize) }
    }
}

impl<const SIZE: usize, T: Copy> Default for InlineBuffer<SIZE, T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct InlineIter<'a, const SIZE: usize, T: Copy = Move> {
    len: u8,
    cur: u8,
    v: &'a [MaybeUninit<T>; SIZE],
}

impl<'a, const SIZE: usize, T: Copy> Iterator for InlineIter<'a, SIZE, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == self.cur {
            return None;
        }
        let res = unsafe { *self.v.get_unchecked(self.cur as usize).as_ptr() };
        self.cur += 1;
        Some(res)
    }
}
