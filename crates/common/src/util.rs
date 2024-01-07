use std::ops::{Add, BitXor, Neg};

#[inline]
pub fn cond_flip<N>(v: N, cond: bool) -> N
where
    N: BitXor<Output = N> + Neg<Output = N> + Add<Output = N> + From<bool>,
{
    (v ^ -N::from(cond)) + N::from(cond)
}
