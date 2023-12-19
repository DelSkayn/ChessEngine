use std::mem::MaybeUninit;

use crate::{gen2::MoveList, Move};

const MAX_MOVES: usize = 218;

/// A constant size buffer stored on the stack,
/// Can be used for storing moves without allocation.
#[derive(Copy)]
pub struct InlineBuffer<const SIZE: usize = MAX_MOVES, T: Copy = Move> {
    moves: [MaybeUninit<T>; SIZE],
    len: u16,
}

impl<const SIZE: usize, T: Copy> Clone for InlineBuffer<SIZE, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<const SIZE: usize, T: Copy> InlineBuffer<SIZE, T> {
    #[inline]
    pub fn new() -> Self {
        debug_assert!(SIZE <= u16::MAX.into());
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
    pub fn swap_remove(&mut self, idx: usize) {
        assert!(
            idx < self.len as usize,
            "got idx: {} while len is {}",
            idx,
            self.len
        );
        self.moves.swap(idx, self.len as usize - 1);
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
    pub fn len(&self) -> usize {
        self.len as usize
    }

    #[inline]
    pub fn truncate(&mut self, len: usize) {
        assert!(len <= self.len as usize);
        self.len = len as u16;
    }

    #[inline]
    pub fn swap(&mut self, a: usize, b: usize) {
        self.moves.swap(a, b);
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<const SIZE: usize, T: Copy> Default for InlineBuffer<SIZE, T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct InlineIter<'a, const SIZE: usize, T: Copy = Move> {
    len: u16,
    cur: u16,
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

impl<const SIZE: usize> MoveList for InlineBuffer<SIZE> {
    fn push(&mut self, m: Move) {
        assert!((self.len as usize) < SIZE);
        self.moves[self.len as usize] = MaybeUninit::new(m);
        self.len += 1;
    }

    fn clear(&mut self) {
        self.len = 0;
    }

    fn len(&self) -> usize {
        self.len as usize
    }
}
