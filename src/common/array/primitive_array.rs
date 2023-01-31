use bitvec::vec::BitVec;

use super::{Array, ArrayBuilder};

pub trait PrimitiveType: Sync + Send + Default + 'static {}

pub type I16Array = PrimitiveArray<i16>;
pub type I32Array = PrimitiveArray<i32>;
pub type I64Array = PrimitiveArray<i64>;
pub type F32Array = PrimitiveArray<f32>;
pub type F64Array = PrimitiveArray<f64>;
pub type BoolArray = PrimitiveArray<bool>;

pub type I16ArrayBuilder = PrimitiveArrayBuilder<i16>;
pub type I32ArrayBuilder = PrimitiveArrayBuilder<i32>;
pub type I64ArrayBuilder = PrimitiveArrayBuilder<i64>;
pub type F32ArrayBuilder = PrimitiveArrayBuilder<f32>;
pub type F64ArrayBuilder = PrimitiveArrayBuilder<f64>;
pub type BoolArrayBuilder = PrimitiveArrayBuilder<bool>;

impl PrimitiveType for i16 {}
impl PrimitiveType for i32 {}
impl PrimitiveType for i64 {}
impl PrimitiveType for f32 {}
impl PrimitiveType for f64 {}
impl PrimitiveType for bool {}

pub struct PrimitiveArray<T: PrimitiveType> {
    data: Vec<T>,
    bitmap: BitVec,
}

impl<T> Array for PrimitiveArray<T>
where
    T: PrimitiveType,
{
    type Builder = PrimitiveArrayBuilder<T>;
    type OwnedItem = T;

    fn get(&self, idx: usize) -> Option<&Self::OwnedItem> {
        todo!()
    }

    fn len(&self) -> usize {
        todo!()
    }

    fn iter(&self) -> super::iterator::ArrayIterator<Self> {
        todo!()
    }
}

pub struct PrimitiveArrayBuilder<T: PrimitiveType> {
    data: Vec<T>,
    bitmap: BitVec,
}

impl<T> ArrayBuilder for PrimitiveArrayBuilder<T>
where
    T: PrimitiveType,
{
    type Array = PrimitiveArray<T>;

    fn with_capacity(capacity: usize) -> Self {
        todo!()
    }

    fn push(&mut self, value: Option<T>) {
        todo!()
    }

    fn finish(self) -> Self::Array {
        todo!()
    }
}
