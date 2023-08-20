use std::sync::Arc;

use ry_filesystem::location::Location;
use ry_name_resolution::ModuleScope;
use ry_stable_likely::unlikely;
use ry_thir::generic_parameter_scope::{GenericParameterData, GenericParameterScope};
use ry_thir::ty::Type;
use ry_thir::Predicate;

use crate::diagnostics::{BoundsInTypeAliasDiagnostic, DuplicateGenericParameterDiagnostic};
use crate::TypeCheckingContext;

impl TypeCheckingContext<'_, '_, '_> {
    pub(crate) fn analyze_generic_parameters_and_where_predicates(
        &self,
        is_type_alias: bool,
        module_item_name_location: Location,
        parent_generic_parameter_scope: Option<Arc<GenericParameterScope>>,
        generic_parameters_hir: &[ry_hir::GenericParameter],
        where_predicates_hir: &[ry_hir::WherePredicate],
        module_scope: &ModuleScope,
    ) -> Option<(GenericParameterScope, Vec<Predicate>)> {
        let (generic_parameter_scope, mut predicates) = self.analyze_generic_parameters(
            is_type_alias,
            module_item_name_location,
            parent_generic_parameter_scope,
            generic_parameters_hir,
            module_scope,
        )?;

        self.analyze_where_predicates(
            &generic_parameter_scope,
            where_predicates_hir,
            &mut predicates,
            module_scope,
        );

        Some((generic_parameter_scope, predicates))
    }

    pub(crate) fn analyze_generic_parameters(
        &self,
        is_type_alias: bool,
        module_item_name_location: Location,
        parent_generic_parameter_scope: Option<Arc<GenericParameterScope>>,
        generic_parameters_hir: &[ry_hir::GenericParameter],
        module_scope: &ModuleScope,
    ) -> Option<(GenericParameterScope, Vec<Predicate>)> {
        let mut generic_parameter_scope =
            GenericParameterScope::new(parent_generic_parameter_scope);
        let mut predicates = Vec::new();

        for parameter_hir in generic_parameters_hir {
            if let Some(data) = generic_parameter_scope.resolve(parameter_hir.name.id) {
                self.diagnostics.write().add_single_file_diagnostic(
                    module_item_name_location.file_path_id,
                    DuplicateGenericParameterDiagnostic::new(
                        data.location,
                        parameter_hir.name.location,
                        self.identifier_interner
                            .resolve(parameter_hir.name.id)
                            .unwrap(),
                    ),
                );
            }

            let default_value =
                parameter_hir
                    .default_value
                    .as_ref()
                    .and_then(|default_value_hir| {
                        self.resolve_type(default_value_hir, &generic_parameter_scope, module_scope)
                    });

            if let Some(bounds_hir) = &parameter_hir.bounds {
                self.analyze_generic_parameter_bounds(
                    is_type_alias,
                    module_item_name_location,
                    parameter_hir,
                    bounds_hir,
                    &generic_parameter_scope,
                    &mut predicates,
                    module_scope,
                );
            }

            generic_parameter_scope.add_generic_parameter(
                parameter_hir.name.id,
                GenericParameterData {
                    location: parameter_hir.name.location,
                    default_value,
                },
            )
        }

        Some((generic_parameter_scope, predicates))
    }

    pub(crate) fn analyze_generic_parameter_bounds(
        &self,
        is_type_alias: bool,
        module_item_name_location: Location,
        parameter_hir: &ry_hir::GenericParameter,
        bounds_hir: &[ry_hir::TypeConstructor],
        generic_parameter_scope: &GenericParameterScope,
        predicates: &mut Vec<Predicate>,
        module_scope: &ModuleScope,
    ) {
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

            return;
        }

        let mut bounds = vec![];

        for bound_hir in bounds_hir {
            let Some(bound) =
                self.resolve_interface(bound_hir.clone(), &generic_parameter_scope, module_scope)
            else {
                continue;
            };

            bounds.push(bound);
        }

        predicates.push(Predicate {
            ty: Type::new_primitive(parameter_hir.name.id),
            bounds,
        })
    }

    pub(crate) fn analyze_where_predicates(
        &self,
        generic_parameter_scope: &GenericParameterScope,
        where_predicates_hir: &[ry_hir::WherePredicate],
        predicates: &mut Vec<Predicate>,
        module_scope: &ModuleScope,
    ) {
        for where_predicate_hir in where_predicates_hir {
            let Some(ty) = self.resolve_type(
                &where_predicate_hir.ty,
                generic_parameter_scope,
                module_scope,
            ) else {
                continue;
            };

            let bounds = self.resolve_bounds(
                generic_parameter_scope,
                &where_predicate_hir.bounds,
                module_scope,
            );

            predicates.push(Predicate { ty, bounds });
        }
    }
}
