use std::sync::Arc;

use ry_name_resolution::{ModuleScope, NameBinding, NameBindingKind, Path};
use ry_thir::{GeneralTypeSignature, ModuleItemSignature};

use crate::TypeCheckingContext;

pub mod signature_analysis_context;
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
        let definition_id = name_binding.definition_id_or_panic();

        match name_binding.kind() {
            NameBindingKind::TypeAlias => {
                self.analyze_type_alias_signature(definition_id, module_scope)
            }
            _ => unreachable!(),
        }
    }
}
