use crate::name::Path;

use super::{RawType, TypeAnnotations};

#[derive(Debug, PartialEq)]
pub struct PrimaryType {
    pub path: Path,
    pub type_annotations: TypeAnnotations,
}

impl From<PrimaryType> for RawType {
    fn from(primary: PrimaryType) -> Self {
        Self::Primary(primary)
    }
}
