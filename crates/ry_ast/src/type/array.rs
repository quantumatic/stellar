use super::{RawType, Type};

#[derive(Debug, PartialEq)]
pub struct ArrayType {
    inner: Type,
}

impl ArrayType {
    #[inline]
    pub const fn inner(&self) -> &Type {
        &self.inner
    }
}

impl From<ArrayType> for RawType {
    fn from(array: ArrayType) -> Self {
        Self::Array(array)
    }
}
