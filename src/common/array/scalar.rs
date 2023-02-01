use super::Array;

mod impls;

pub trait Scalar:
    Sync + Send + Clone + std::fmt::Debug + 'static + TryFrom<ScalarImpl> + Into<ScalarImpl>
{
    type ArrayType: Array<OwnedItem = Self>;
    type RefType<'a>: ScalarRef<'a, ScalarType = Self, ArrayType = Self::ArrayType>;
    fn as_scalar_ref(&self) -> Self::RefType<'_>;
}

pub trait ScalarRef<'a>:
    Sync
    + Send
    + Clone
    + Copy
    + 'a
    + std::fmt::Debug
    + TryFrom<ScalarRefImpl<'a>>
    + Into<ScalarRefImpl<'a>>
{
    type ArrayType: Array<ItemRef<'a> = Self>;
    type ScalarType: Scalar<RefType<'a> = Self>;

    fn to_owned_scalar(&self) -> Self::ScalarType;
}

pub enum ScalarImpl {
    Int16(i16),
    Int32(i32),
    Int64(i64),
    // Float32(f32),
    // Float64(f64),
    Bool(bool),
    String(String),
}

pub enum ScalarRefImpl<'a> {
    Int16(i16),
    Int32(i32),
    Int64(i64),
    // Float32(f32),
    // Float64(f64),
    Bool(bool),
    String(&'a str),
}
