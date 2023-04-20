use super::Item;
use crate::{name::Path, Visibility};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ImportItem {
    pub visibility: Visibility,
    pub path: Path,
}

impl From<ImportItem> for Item {
    fn from(import: ImportItem) -> Self {
        Self::Import(import)
    }
}
