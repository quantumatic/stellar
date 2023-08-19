use std::collections::hash_map;

use ry_fx_hash::FxHashMap;
use ry_name_resolution::DefinitionID;

use crate::ModuleItemState;

/// Storage of THIR for module items.
#[derive(Debug, Default)]
pub struct THIRStorage {
    module_items: FxHashMap<DefinitionID, ry_thir::ModuleItem>,
}

impl IntoIterator for THIRStorage {
    type IntoIter = hash_map::IntoIter<DefinitionID, ry_thir::ModuleItem>;
    type Item = (DefinitionID, ry_thir::ModuleItem);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.module_items.into_iter()
    }
}

impl THIRStorage {
    /// Creates an empty THIR storage.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Resolves a module item.
    #[inline]
    pub fn resolve_module_item(&self, definition_id: DefinitionID) -> Option<&ry_thir::ModuleItem> {
        self.module_items.get(&definition_id)
    }

    /// Resolves a module item.
    ///
    /// # Panics
    /// If the module item doesn't exist in the storage.
    pub fn resolve_module_item_or_panic(
        &self,
        definition_id: DefinitionID,
    ) -> &ry_thir::ModuleItem {
        self.resolve_module_item(definition_id)
            .unwrap_or_else(|| panic!("cannot resolve THIR for {:?}", definition_id))
    }

    /// Adds a module item to the storage.
    #[inline]
    pub fn add_item(&mut self, definition_id: DefinitionID, hir: ry_thir::ModuleItem) {
        self.module_items.insert(definition_id, hir);
    }

    /// Extends the storage with new items.
    #[inline]
    pub fn extend(&mut self, items: impl IntoIterator<Item = (DefinitionID, ry_thir::ModuleItem)>) {
        self.module_items.extend(items);
    }

    /// Removes a module item from the storage.
    #[inline]
    pub fn remove_item(&mut self, definition_id: DefinitionID) {
        self.module_items.remove(&definition_id);
    }
}
