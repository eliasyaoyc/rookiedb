mod cmp;
mod string;

use std::marker::PhantomData;

use anyhow::Result;

use super::*;
use crate::common::array::scalar::Scalar;

pub struct BinaryExpression<I1: Array, I2: Array, O: Array, F> {
    func: F,
    _phantom: PhantomData<(I1, I2, O)>,
}

impl<'a, I1: Array, I2: Array, O: Array, F> BinaryExpression<I1, I2, O, F>
where
    &'a I1: TryFrom<&'a ArrayImpl, Error = TypeMismatch>,
    &'a I2: TryFrom<&'a ArrayImpl, Error = TypeMismatch>,
    F: Fn(I1::ItemRef<'a>, I2::ItemRef<'a>) -> O::OwnedItem,
{
    pub fn new(func: F) -> Self {
        Self {
            func,
            _phantom: PhantomData,
        }
    }

    pub fn eval(&self, i1: &'a ArrayImpl, i2: &'a ArrayImpl) -> Result<ArrayImpl> {
        let i1a: &'a I1 = i1.try_into()?;
        let i2a: &'a I2 = i2.try_into()?;
        assert_eq!(i1.len(), i2.len(), "array length mismatch");
        let mut builder: O::Builder = O::Builder::with_capacity(i1.len());
        for (i1, i2) in i1a.iter().zip(i2a.iter()) {
            match (i1, i2) {
                (Some(i1), Some(i2)) => builder.push(Some((self.func)(i1, i2).as_scalar_ref())),
                _ => builder.push(None),
            }
        }
        Ok(builder.finish().into())
    }
}

#[cfg(test)]
mod tests {
    use super::{cmp::*, string::*, *};
    use crate::common::array::string_array::StringArray;

    /// Test if an array has the same content as a vector
    fn check_array_eq<'a, A: Array>(array: &'a A, vec: &[Option<A::ItemRef<'a>>])
    where
        A::ItemRef<'a>: PartialEq,
    {
        for (a, b) in array.iter().zip(vec.iter()) {
            assert_eq!(&a, b);
        }
    }

    #[test]
    fn test_cmp_le() {
        // Compare two `i32` array. Cast them to `i64` before comparing.
        let expr = BinaryExpression::<I32Array, I32Array, BoolArray, _>::new(
            cmp_le::<I32Array, I32Array, I64Array>,
        );
        let result = expr
            .eval(
                &I32Array::from_slice(&[Some(0), Some(1), None]).into(),
                &I32Array::from_slice(&[Some(1), Some(0), None]).into(),
            )
            .unwrap();
        check_array_eq::<BoolArray>(
            (&result).try_into().unwrap(),
            &[Some(true), Some(false), None],
        );
    }

    #[test]
    fn test_cmp_ge_str() {
        let expr = BinaryExpression::<StringArray, StringArray, BoolArray, _>::new(
            cmp_ge::<StringArray, StringArray, StringArray>,
        );
        let result = expr
            .eval(
                &StringArray::from_slice(&[Some("0"), Some("1"), None]).into(),
                &StringArray::from_slice(&[Some("1"), Some("0"), None]).into(),
            )
            .unwrap();
        check_array_eq::<BoolArray>(
            (&result).try_into().unwrap(),
            &[Some(false), Some(true), None],
        );
    }

    #[test]
    fn test_str_contains() {
        let expr = BinaryExpression::<StringArray, StringArray, BoolArray, _>::new(str_contains);
        let result = expr
            .eval(
                &StringArray::from_slice(&[Some("000"), Some("111"), None]).into(),
                &StringArray::from_slice(&[Some("0"), Some("0"), None]).into(),
            )
            .unwrap();
        check_array_eq::<BoolArray>(
            (&result).try_into().unwrap(),
            &[Some(true), Some(false), None],
        );
    }
}
