mod expr;
mod impls;
mod iterator;
mod macros;
mod primitive_array;
mod scalar;
mod string_array;

use thiserror::Error;

use self::{
    iterator::ArrayIterator,
    string_array::{StringArray, StringArrayBuilder},
};

#[derive(Debug, Error)]
#[error("Type mismatch on conversion: expected {0}, got {1}")]
pub struct TypeMismatch(&'static str, &'static str);

pub trait Array: Sync + Send + Sized + 'static {
    type Builder: ArrayBuilder<Array = Self>;
    type OwnedItem;

    fn get(&self, idx: usize) -> Option<&Self::OwnedItem>;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn iter(&self) -> ArrayIterator<Self>;
}

pub trait ArrayBuilder {
    type Array: Array<Builder = Self>;

    fn with_capacity(capacity: usize) -> Self;

    fn push(&mut self, value: Option<<Self::Array as Array>::OwnedItem>);

    fn finish(self) -> Self::Array;
}

pub enum ArrayImpl {
    Int16(),
    Int32(),
    Int64(),
    Float32(),
    Float64(),
    Bool(),
    String(StringArray),
}

pub enum ArrayBuilderImpl {
    Int16(),
    Int32(),
    Int64(),
    Float32(),
    Float64(),
    Bool(),
    String(StringArrayBuilder),
}
