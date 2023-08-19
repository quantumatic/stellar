use std::sync::Arc;

use ry_name_resolution::{ModuleScope, NameBinding};
use ry_thir::ModuleItemSignature;

use crate::TypeCheckingContext;

mod type_alias;

impl TypeCheckingContext<'_, '_, '_> {
    pub(crate) fn resolve_signature(
        &self,
        name_binding: NameBinding,
        module_scope: &ModuleScope,
    ) -> Option<Arc<ModuleItemSignature>> {
        if let Some(signature) = self
            .signatures
            .get(&name_binding.definition_id_or_panic())
            .cloned()
        {
            Some(signature)
        } else {
            self.analyze_signature(name_binding, module_scope)
        }
    }

    pub(crate) fn analyze_signature(
        &self,
        name_binding: NameBinding,
        module_scope: &ModuleScope,
    ) -> Option<Arc<ModuleItemSignature>> {
        match name_binding {
            NameBinding::TypeAlias(definition_id) => {
                self.analyze_type_alias_signature(definition_id, module_scope)
            }
            _ => unreachable!(),
        }
    }
}
