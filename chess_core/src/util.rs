//! Various board array utilites.

use crate::{Direction, Piece, Square};
use std::ops::{Index, IndexMut};

/// An array the size of a board.
/// Can be effeciently indexed by a square.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct BoardArray<T>([T; 64]);

impl<T> BoardArray<T> {
    pub const fn new_array(t: [T; 64]) -> Self {
        BoardArray(t)
    }
}

impl<T: Copy> BoardArray<T> {
    pub fn new(t: T) -> Self {
        BoardArray([t; 64])
    }
}

impl<T: Copy> Index<Square> for BoardArray<T> {
    type Output = T;

    #[inline(always)]
    fn index(&self, index: Square) -> &Self::Output {
        unsafe { self.0.get_unchecked(index.get() as usize) }
    }
}

impl<T: Copy> IndexMut<Square> for BoardArray<T> {
    #[inline(always)]
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        unsafe { self.0.get_unchecked_mut(index.get() as usize) }
    }
}

/// An array the size of all possible directions.
/// Can be effeciently indexed by direction.
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

/// An array the size of all possible pieces.
/// Can be effeciently indexed by Piece.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct PieceArray<T>([T; 12]);

impl<T: Copy> PieceArray<T> {
    pub fn new(t: T) -> Self {
        PieceArray([t; 12])
    }
}

impl<T> PieceArray<T> {
    pub const fn new_array(t: [T; 12]) -> Self {
        PieceArray(t)
    }
}

impl<T> Index<Piece> for PieceArray<T> {
    type Output = T;

    #[inline(always)]
    fn index(&self, index: Piece) -> &Self::Output {
        unsafe { self.0.get_unchecked(index as usize) }
    }
}

impl<T> IndexMut<Piece> for PieceArray<T> {
    #[inline(always)]
    fn index_mut(&mut self, index: Piece) -> &mut Self::Output {
        unsafe { self.0.get_unchecked_mut(index as usize) }
    }
}
