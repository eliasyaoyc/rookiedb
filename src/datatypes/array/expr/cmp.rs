use std::cmp::Ordering;

use crate::datatypes::array::Array;

pub fn cmp_le<'a, I1: Array, I2: Array, C: Array + 'static>(
    i1: I1::ItemRef<'a>,
    i2: I2::ItemRef<'a>,
) -> bool
where
    I1::ItemRef<'a>: Into<C::ItemRef<'a>>,
    I2::ItemRef<'a>: Into<C::ItemRef<'a>>,
    C::ItemRef<'a>: PartialOrd,
{
    i1.into().partial_cmp(&i2.into()).unwrap() == Ordering::Less
}

pub fn cmp_ge<'a, I1: Array, I2: Array, C: Array + 'static>(
    i1: I1::ItemRef<'a>,
    i2: I2::ItemRef<'a>,
) -> bool
where
    I1::ItemRef<'a>: Into<C::ItemRef<'a>>,
    I2::ItemRef<'a>: Into<C::ItemRef<'a>>,
    C::ItemRef<'a>: PartialOrd,
{
    i1.into().partial_cmp(&i2.into()).unwrap() == Ordering::Greater
}