use std::collections::hash_map;

use ry_fx_hash::FxHashMap;
use ry_name_resolution::DefinitionID;

/// Storage of HIR for module items.
#[derive(Debug, Default)]
pub struct HIRStorage {
    module_items: FxHashMap<DefinitionID, ry_hir::ModuleItem>,
}

impl IntoIterator for HIRStorage {
    type Item = (DefinitionID, ry_hir::ModuleItem);
    type IntoIter = hash_map::IntoIter<DefinitionID, ry_hir::ModuleItem>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.module_items.into_iter()
    }
}

impl HIRStorage {
    /// Creates an empty HIR storage.
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Resolves a module item.
    #[inline(always)]
    pub fn resolve_module_item(&self, definition_id: DefinitionID) -> Option<&ry_hir::ModuleItem> {
        self.module_items.get(&definition_id)
    }

    /// Resolves a module item.
    ///
    /// # Panics
    /// If the module item doesn't exist in the storage.
    #[inline(always)]
    pub fn resolve_module_item_or_panic(&self, definition_id: DefinitionID) -> &ry_hir::ModuleItem {
        self.resolve_module_item(definition_id)
            .unwrap_or_else(|| panic!("cannot resolve HIR for {:?}", definition_id))
    }

    /// Resolves a type alias HIR.
    #[inline(always)]
    pub fn resolve_type_alias(&self, definition_id: DefinitionID) -> Option<&ry_hir::TypeAlias> {
        match self.resolve_module_item(definition_id)? {
            ry_hir::ModuleItem::TypeAlias(alias) => Some(alias),
            _ => None,
        }
    }

    /// Resolves a type alias HIR.
    ///
    /// # Panics
    /// If the type alias doesn't exist in the storage.
    #[inline(always)]
    pub fn resolve_type_alias_or_panic(&self, definition_id: DefinitionID) -> &ry_hir::TypeAlias {
        self.resolve_type_alias(definition_id)
            .unwrap_or_else(|| panic!("cannot resolve type alias HIR for {:?}", definition_id))
    }

    /// Adds a module item to the storage.
    #[inline(always)]
    pub fn add_module_item(&mut self, definition_id: DefinitionID, hir: ry_hir::ModuleItem) {
        self.module_items.insert(definition_id, hir);
    }

    /// Extends the storage with new items.
    #[inline(always)]
    pub fn extend(&mut self, items: impl IntoIterator<Item = (DefinitionID, ry_hir::ModuleItem)>) {
        self.module_items.extend(items);
    }

    /// Removes a module item from the storage.
    #[inline(always)]
    pub fn remove_module_item(&mut self, definition_id: DefinitionID) {
        self.module_items.remove(&definition_id);
    }
}
