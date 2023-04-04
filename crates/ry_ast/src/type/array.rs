use super::{RawType, Type};

#[derive(Debug, PartialEq)]
pub struct ArrayType {
    pub inner: Box<Type>,
}

impl From<ArrayType> for RawType {
    fn from(array: ArrayType) -> Self {
        Self::Array(array)
    }
}
