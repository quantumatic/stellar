use ry_fx_hash::FxHashMap;
use ry_name_resolution::DefinitionID;

use crate::ModuleItemState;

/// Storage of THIR for module items.
#[derive(Debug, Default)]
pub struct THIRStorage {
    module_items: FxHashMap<DefinitionID, ry_thir::ModuleItem>,
}

impl THIRStorage {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn resolve_module_item(&self, definition_id: DefinitionID) -> Option<&ry_thir::ModuleItem> {
        self.module_items.get(&definition_id)
    }

    pub fn resolve_module_item_or_panic(
        &self,
        definition_id: DefinitionID,
    ) -> &ry_thir::ModuleItem {
        self.resolve_module_item(definition_id)
            .unwrap_or_else(|| panic!("cannot resolve THIR for {:?}", definition_id))
    }

    #[inline]
    pub fn add_item(&mut self, definition_id: DefinitionID, hir: ry_thir::ModuleItem) {
        self.module_items.insert(definition_id, hir);
    }
}
