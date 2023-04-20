use super::{RawType, Type};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ArrayType {
    pub inner: Box<Type>,
}

impl From<ArrayType> for RawType {
    fn from(array: ArrayType) -> Self {
        Self::Array(array)
    }
}
