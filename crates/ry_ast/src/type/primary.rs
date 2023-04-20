use super::{RawType, TypeAnnotations};
use crate::name::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PrimaryType {
    pub path: Path,
    pub type_annotations: TypeAnnotations,
}

impl From<PrimaryType> for RawType {
    fn from(primary: PrimaryType) -> Self {
        Self::Primary(primary)
    }
}
