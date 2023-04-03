use super::{RawType, Type};
use crate::Mutability;

#[derive(Debug, PartialEq)]
pub struct ReferenceType {
    pub mutability: Mutability,
    pub inner: Box<Type>,
}

impl ReferenceType {
    #[inline]
    pub fn new(mutability: Mutability, inner: Type) -> Self {
        Self {
            mutability,
            inner: Box::new(inner),
        }
    }
}

impl From<ReferenceType> for RawType {
    fn from(reference: ReferenceType) -> Self {
        Self::Reference(reference)
    }
}
