use crate::{name::Path, Visibility};

use super::Item;

#[derive(Debug, PartialEq)]
pub struct ImportItem {
    pub visibility: Visibility,
    pub path: Path,
}

impl From<ImportItem> for Item {
    fn from(import: ImportItem) -> Self {
        Item::Import(import)
    }
}
