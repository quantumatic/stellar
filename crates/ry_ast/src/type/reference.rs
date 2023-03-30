use crate::span::Span;

use super::{RawType, Type};

#[derive(Debug, PartialEq)]
pub struct ReferenceType {
    mutable: Option<Span>,
    inner: Type,
}

impl ReferenceType {
    #[inline]
    pub const fn mutable(&self) -> Option<&Span> {
        self.mutable.as_ref()
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
