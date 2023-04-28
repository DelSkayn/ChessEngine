//! Various board array utilites.

use crate::{Direction, Piece, Square};
use std::ops::{Deref, Index, IndexMut};

#[cfg(feature = "serde")]
mod arrays {
    use std::{marker::PhantomData, mem::MaybeUninit};

    use serde::{
        de::{SeqAccess, Visitor},
        ser::SerializeTuple,
        Deserialize, Deserializer, Serialize, Serializer,
    };
    pub fn serialize<S: Serializer, T: Serialize, const N: usize>(
        data: &[T; N],
        ser: S,
    ) -> Result<S::Ok, S::Error> {
        let mut s = ser.serialize_tuple(N)?;
        for item in data {
            s.serialize_element(item)?;
        }
        s.end()
    }

    struct ArrayVisitor<T, const N: usize>(PhantomData<T>);

    impl<'de, T: Sized, const N: usize> Visitor<'de> for ArrayVisitor<T, N>
    where
        T: Deserialize<'de>,
    {
        type Value = [T; N];

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str(&format!("an array of length {}", N))
        }

        #[inline]
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            // can be optimized using MaybeUninit
            let mut data: MaybeUninit<[T; N]> = MaybeUninit::uninit();
            let ptr = data.as_mut_ptr() as *mut T;
            for i in 0..N {
                match (seq.next_element())? {
                    Some(val) => unsafe {
                        ptr.add(i).write(val);
                    },
                    None => return Err(serde::de::Error::invalid_length(N, &self)),
                }
            }
            unsafe { Ok(data.assume_init()) }
        }
    }
    pub fn deserialize<'de, D, T, const N: usize>(deserializer: D) -> Result<[T; N], D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        deserializer.deserialize_tuple(N, ArrayVisitor::<T, N>(PhantomData))
    }
}

/// An array the size of a board.
/// Can be effeciently indexed by a square.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "T: serde::Serialize",
        deserialize = "T: serde::Deserialize<'de>"
    ))
)]
pub struct BoardArray<T>(#[cfg_attr(feature = "serde", serde(with = "arrays"))] [T; 64]);

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

impl<T> Deref for BoardArray<T> {
    type Target = [T; 64];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Index<Square> for BoardArray<T> {
    type Output = T;

    #[inline(always)]
    fn index(&self, index: Square) -> &Self::Output {
        unsafe { self.0.get_unchecked(index.get() as usize) }
    }
}

impl<T> IndexMut<Square> for BoardArray<T> {
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
