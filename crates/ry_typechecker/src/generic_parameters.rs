use std::sync::Arc;

use ry_diagnostics::BuildDiagnostic;
use ry_filesystem::location::Location;
use ry_name_resolution::ModuleScope;
use ry_stable_likely::unlikely;
use ry_thir::generic_parameter_scope::{GenericParameterData, GenericParameterScope};
use ry_thir::ty::Type;
use ry_thir::Predicate;

use crate::diagnostics::BoundsInTypeAliasDiagnostic;
use crate::TypeCheckingContext;

impl TypeCheckingContext<'_, '_, '_> {
    pub(crate) fn analyze_generic_parameters_and_bounds(
        &self,
        is_type_alias: bool,
        module_item_name_location: Location,
        parent_generic_parameter_scope: Option<Arc<GenericParameterScope>>,
        generic_parameters_hir: &[ry_hir::GenericParameter],
        where_predicates_hir: &[ry_hir::WherePredicate],
        module_scope: &ModuleScope,
    ) -> Option<(GenericParameterScope, Vec<Predicate>)> {
        let mut generic_parameter_scope =
            GenericParameterScope::new(parent_generic_parameter_scope);
        let mut predicates = Vec::new();

        for parameter_hir in generic_parameters_hir {
            if let Some(data) = generic_parameter_scope.resolve(parameter_hir.name.id) {
                // emit diagnostics
            }

            let default_value = if let Some(default_value_hir) = &parameter_hir.default_value {
                if let Some(ty) = self.resolve_type(
                    default_value_hir,
                    Some(&generic_parameter_scope),
                    module_scope,
                ) {
                    Some(ty)
                } else {
                    return None;
                }
            } else {
                None
            };

            if let Some(bounds_hir) = &parameter_hir.bounds {
                if unlikely(is_type_alias) {
                    self.diagnostics.write().add_single_file_diagnostic(
                        module_item_name_location.file_path_id,
                        BoundsInTypeAliasDiagnostic::new(
                            module_item_name_location,
                            Location {
                                start: bounds_hir.first().unwrap().location.start,
                                end: bounds_hir.last().unwrap().location.end,
                                file_path_id: module_item_name_location.file_path_id,
                            },
                        ),
                    );

                    return None;
                }

                let mut bounds = vec![];

                for bound_hir in bounds_hir {
                    bounds.push(self.resolve_interface(
                        bound_hir.clone(),
                        Some(&generic_parameter_scope),
                        module_scope,
                    )?);
                }

                predicates.push(Predicate {
                    ty: Type::new_primitive(parameter_hir.name.id),
                    bounds,
                })
            }

            generic_parameter_scope.add_generic_parameter(
                parameter_hir.name.id,
                GenericParameterData {
                    location: parameter_hir.name.location,
                    default_value,
                },
            )
        }

        for where_predicate_hir in where_predicates_hir {
            let ty = self.resolve_type(
                &where_predicate_hir.ty,
                Some(&generic_parameter_scope),
                module_scope,
            )?;
        }

        Some((generic_parameter_scope, predicates))
    }
}
