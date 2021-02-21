use super::Direction;
use crate::Square;
use std::ops::{Index, IndexMut};

#[derive(Clone, Copy)]
pub struct BoardArray<T: Copy>([T; 64]);

impl<T: Copy> BoardArray<T> {
    pub fn new(t: T) -> Self {
        BoardArray([t; 64])
    }
}

impl<T: Copy> Index<Square> for BoardArray<T> {
    type Output = T;

    fn index(&self, index: Square) -> &Self::Output {
        unsafe { self.0.get_unchecked(index.get() as usize) }
    }
}

impl<T: Copy> IndexMut<Square> for BoardArray<T> {
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        unsafe { self.0.get_unchecked_mut(index.get() as usize) }
    }
}

#[derive(Clone, Copy)]
pub struct DirectionArray<T: Copy>([T; 8]);

impl<T: Copy> DirectionArray<T> {
    pub fn new(t: T) -> Self {
        DirectionArray([t; 8])
    }
}

impl<T: Copy> Index<Direction> for DirectionArray<T> {
    type Output = T;

    fn index(&self, index: Direction) -> &Self::Output {
        unsafe { self.0.get_unchecked(index as usize) }
    }
}

impl<T: Copy> IndexMut<Direction> for DirectionArray<T> {
    fn index_mut(&mut self, index: Direction) -> &mut Self::Output {
        unsafe { self.0.get_unchecked_mut(index as usize) }
    }
}
