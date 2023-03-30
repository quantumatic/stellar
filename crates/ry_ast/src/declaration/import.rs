use string_interner::DefaultSymbol;

use crate::span::WithSpan;

use super::Item;

#[derive(Debug, PartialEq)]
pub struct ImportItem {
    path: WithSpan<Vec<DefaultSymbol>>,
}

impl ImportItem {
    #[inline]
    pub const fn path(&self) -> &WithSpan<Vec<DefaultSymbol>> {
        &self.path
    }
}

impl From<ImportItem> for Item {
    fn from(import: ImportItem) -> Self {
        Item::Import(import)
    }
}
