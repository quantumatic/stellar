use super::{RawType, Type};

#[derive(Debug, PartialEq)]
pub struct ArrayType {
    inner: Box<Type>,
}

impl ArrayType {
    #[inline]
    pub fn new(inner: Type) -> Self {
        Self {
            inner: Box::new(inner),
        }
    }
}

impl From<ArrayType> for RawType {
    fn from(array: ArrayType) -> Self {
        Self::Array(array)
    }
}
