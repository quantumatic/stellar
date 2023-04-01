use crate::name::Path;

use super::Item;

#[derive(Debug, PartialEq)]
pub struct ImportItem {
    path: Path,
}

impl ImportItem {
    #[inline]
    pub const fn new(path: Path) -> Self {
        Self { path }
    }

    #[inline]
    pub const fn path(&self) -> &Path {
        &self.path
    }
}

impl From<ImportItem> for Item {
    fn from(import: ImportItem) -> Self {
        Item::Import(import)
    }
}
