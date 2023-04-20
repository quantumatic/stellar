use super::{RawType, Type};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ReferenceType {
    pub inner: Box<Type>,
}

impl From<ReferenceType> for RawType {
    fn from(reference: ReferenceType) -> Self {
        Self::Reference(reference)
    }
}
