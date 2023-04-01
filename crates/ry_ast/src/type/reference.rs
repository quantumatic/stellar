use super::{RawType, Type};
use crate::Mutability;

#[derive(Debug, PartialEq)]
pub struct ReferenceType {
    mutability: Mutability,
    inner: Type,
}

impl ReferenceType {
    pub const fn new(mutability: Mutability, inner: Type) -> Self {
        Self { mutability, inner }
    }

    #[inline]
    pub const fn mutability(&self) -> Mutability {
        self.mutability
    }

    #[inline]
    pub const fn inner(&self) -> &Type {
        &self.inner
    }
}

impl From<ReferenceType> for RawType {
    fn from(reference: ReferenceType) -> Self {
        Self::Reference(reference)
    }
}
