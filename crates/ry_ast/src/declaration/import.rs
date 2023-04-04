use crate::name::Path;

use super::Item;

#[derive(Debug, PartialEq)]
pub struct ImportItem {
    pub path: Path,
}

impl From<ImportItem> for Item {
    fn from(import: ImportItem) -> Self {
        Item::Import(import)
    }
}
