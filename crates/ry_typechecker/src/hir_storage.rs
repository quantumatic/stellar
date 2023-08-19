use ry_fx_hash::FxHashMap;
use ry_name_resolution::DefinitionID;

use crate::ModuleItemState;

/// Storage of HIR for module items.
#[derive(Debug, Default)]
pub struct HIRStorage {
    module_items: FxHashMap<DefinitionID, ry_hir::ModuleItem>,
}

impl HIRStorage {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn resolve_module_item_hir(
        &self,
        definition_id: DefinitionID,
    ) -> Option<&ry_hir::ModuleItem> {
        self.module_items.get(&definition_id)
    }

    pub fn resolve_module_item_hir_or_panic(
        &self,
        definition_id: DefinitionID,
    ) -> &ry_hir::ModuleItem {
        self.resolve_module_item_hir(definition_id)
            .unwrap_or_else(|| panic!("cannot resolve HIR for {:?}", definition_id))
    }

    #[inline]
    pub fn resolve_type_alias_hir(
        &self,
        definition_id: DefinitionID,
    ) -> Option<&ry_hir::TypeAlias> {
        match self.resolve_module_item_hir(definition_id)? {
            ry_hir::ModuleItem::TypeAlias(alias) => Some(alias),
            _ => None,
        }
    }

    #[inline]
    pub fn resolve_type_alias_hir_or_panic(
        &self,
        definition_id: DefinitionID,
    ) -> &ry_hir::TypeAlias {
        self.resolve_type_alias_hir(definition_id)
            .unwrap_or_else(|| panic!("cannot resolve type alias HIR for {:?}", definition_id))
    }

    #[inline]
    pub fn add_module_item(&mut self, definition_id: DefinitionID, hir: ry_hir::ModuleItem) {
        self.module_items.insert(definition_id, hir);
    }

    #[inline]
    pub fn remove_module_item(&mut self, definition_id: DefinitionID) {
        self.module_items.remove(&definition_id);
    }
}
