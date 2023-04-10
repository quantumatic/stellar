use super::{RawType, Type};

#[derive(Debug, PartialEq)]
pub struct ReferenceType {
    pub inner: Box<Type>,
}

impl From<ReferenceType> for RawType {
    fn from(reference: ReferenceType) -> Self {
        Self::Reference(reference)
    }
}
