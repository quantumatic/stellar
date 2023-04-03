use crate::name::Path;

use super::{RawType, Type, TypeAnnotations};

#[derive(Debug, PartialEq)]
pub struct PrimaryType {
    pub name: Path,
    pub type_annotations: TypeAnnotations,
}

impl PrimaryType {
    #[inline]
    pub const fn new(name: Path, type_annotations: Vec<Type>) -> Self {
        Self {
            name,
            type_annotations,
        }
    }
}

impl From<PrimaryType> for RawType {
    fn from(primary: PrimaryType) -> Self {
        Self::Primary(primary)
    }
}
