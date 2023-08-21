use std::sync::Arc;

use ry_name_resolution::{DefinitionID, ModuleScope};
use ry_thir::{ModuleItemSignature, TypeAliasSignature};

use crate::TypeCheckingContext;

impl TypeCheckingContext<'_, '_, '_> {
    /// Analyzes a type alias.
    pub(crate) fn analyze_type_alias_signature(
        &self,
        definition_id: DefinitionID,
        module_scope: &ModuleScope,
    ) -> Option<Arc<ModuleItemSignature>> {
        let hir_storage_reader = self.hir_storage.read();
        let alias = hir_storage_reader.resolve_type_alias_or_panic(definition_id);

        let (generic_parameter_scope, _) = self.analyze_generic_parameters_and_where_predicates(
            true,
            alias.name.location,
            None,
            &alias.generic_parameters,
            &[],
            module_scope,
        )?;

        let value = self.resolve_type(&alias.value, &generic_parameter_scope, module_scope)?;

        Some(Arc::new(ModuleItemSignature::TypeAlias(
            TypeAliasSignature {
                name: alias.name,
                generic_parameter_scope,
                value,
            },
        )))
    }
}
