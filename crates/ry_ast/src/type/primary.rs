use crate::name::Path;

use super::{RawType, Type, TypeAnnotations};

#[derive(Debug, PartialEq)]
pub struct PrimaryType {
    name: Path,
    type_annotations: TypeAnnotations,
}

impl PrimaryType {
    #[inline]
    pub const fn new(name: Path, type_annotations: Vec<Type>) -> Self {
        Self {
            name,
            type_annotations,
        }
    }

    #[inline]
    pub const fn name(&self) -> &Path {
        &self.name
    }

    #[inline]
    pub const fn generics(&self) -> &TypeAnnotations {
        &self.type_annotations
    }
}

impl From<PrimaryType> for RawType {
    fn from(primary: PrimaryType) -> Self {
        Self::Primary(primary)
    }
}
