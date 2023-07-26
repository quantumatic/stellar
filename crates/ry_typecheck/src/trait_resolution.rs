use std::iter::zip;

use ry_ast::{Bounds, DefinitionID, IdentifierAST};
use ry_diagnostics::{BuildDiagnostic, GlobalDiagnostics};
use ry_filesystem::location::Location;
use ry_fx_hash::{FxHashMap, FxHashSet};
use ry_thir::ty::{Path, Type, TypePath};

use crate::diagnostics::ExpectedNominalTypeInInherentImplDiagnostic;

#[derive(Debug, Default)]
pub struct TraitResolutionContext {
    traits: FxHashMap<Path, TraitData>,
    raw_type_implementations: FxHashMap<Path, ImplementationData>,
}

impl TraitResolutionContext {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn get_trait_data(&self, absolute_path: impl AsRef<Path>) -> Option<&TraitData> {
        self.traits.get(absolute_path.as_ref())
    }

    #[inline]
    pub fn get_trait_data_mut(
        &mut self,
        absolute_path: impl AsRef<Path>,
    ) -> Option<&mut TraitData> {
        self.traits.get_mut(absolute_path.as_ref())
    }

    #[inline]
    pub fn get_raw_type_implementation_data(
        &self,
        type_absolute_path: impl AsRef<Path>,
    ) -> Option<&ImplementationData> {
        self.raw_type_implementations
            .get(type_absolute_path.as_ref())
    }

    #[inline]
    pub fn get_raw_type_implementation_data_mut(
        &mut self,
        type_absolute_path: impl AsRef<Path>,
    ) -> Option<&mut ImplementationData> {
        self.raw_type_implementations
            .get_mut(type_absolute_path.as_ref())
    }

    pub fn add_trait_implementation(
        &mut self,
        _trait_path: Path,
        _implementation_data: ImplementationData,
    ) {
        todo!()
    }

    pub fn add_raw_type_implementation(
        &mut self,
        implementation: &ImplementationData,
        diagnostics: &mut GlobalDiagnostics,
    ) {
        let nominal_type = match &implementation.ty {
            Type::WithQualifiedPath { .. } => false,
            Type::Path {
                path: TypePath { segments },
            } => {
                if let [segment] = segments.as_slice() {
                    if let Path { symbols: _ } = &segment.left {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => false,
        };

        if nominal_type {
            diagnostics.add_single_file_diagnostic(
                implementation.definition_id.file_path_id,
                ExpectedNominalTypeInInherentImplDiagnostic {
                    location: implementation.implemented_type_location,
                }
                .build(),
            );
        }
    }

    pub fn check_overlap(&self, trait_path: &Path, implementation_data: &ImplementationData) {
        let Some(candidates) = self.traits.get(trait_path).map(|r#trait| &r#trait.implementations) else {
            return;
        };

        for candidate_data in candidates {
            self.check_overlap_with_candidate(trait_path, candidate_data, implementation_data);
        }
    }

    /// Returns constrained type parameters.
    #[must_use]
    pub fn get_constrained_type_parameters(
        &self,
        implemented_type: Type,
        generics: &FxHashSet<IdentifierAST>,
    ) -> FxHashSet<IdentifierAST> {
        match implemented_type {
            Type::Function {
                parameter_types,
                return_type,
            } => {
                let mut result = FxHashSet::default();

                for parameter_type in parameter_types {
                    result.extend(self.get_constrained_type_parameters(parameter_type, generics));
                }

                result.extend(self.get_constrained_type_parameters(*return_type, generics));

                result
            }
            Type::Unit => FxHashSet::default(),

            // T
            Type::WithQualifiedPath { .. } => FxHashSet::default(),
            Type::Variable(..) => FxHashSet::default(),
            Type::Tuple { element_types } => {
                let mut result = FxHashSet::default();

                for element_type in element_types {
                    result.extend(self.get_constrained_type_parameters(element_type, generics));
                }

                result
            }
            Type::TraitObject { bounds } => {
                let mut result = FxHashSet::default();

                for bound in bounds {
                    for used_generic in &bound.right {
                        result.extend(
                            self.get_constrained_type_parameters(used_generic.clone(), generics),
                        );
                    }
                }

                result
            }
            Type::Path { path } => {
                if let [first_segment] = path.segments.as_slice() {
                    let mut result = FxHashSet::default();

                    if let &[maybe_used_generic] = first_segment.left.symbols.as_slice() {
                        for generic in generics {
                            if generic.symbol == maybe_used_generic {
                                result.insert(*generic);
                            }
                        }
                    }

                    result
                } else {
                    let mut result = FxHashSet::default();

                    for segment in &path.segments {
                        for used_generic in &segment.right {
                            result.extend(
                                self.get_constrained_type_parameters(
                                    used_generic.clone(),
                                    generics,
                                ),
                            );
                        }
                    }

                    result
                }
            }
        }
    }

    /// Get not constrained type parameters - used to validate most of type
    /// implementations.
    #[inline]
    #[must_use]
    pub fn get_not_constrained_type_parameters(
        &self,
        implemented_type: Type,
        type_parameters: &FxHashSet<IdentifierAST>,
    ) -> FxHashSet<IdentifierAST> {
        type_parameters
            .difference(&self.get_constrained_type_parameters(implemented_type, type_parameters))
            .cloned()
            .collect()
    }

    /// The function checks for type equality in implementations. For example
    /// implemented types in `impl[T] T` and `impl[T, M] HashMap[T, M]` can
    /// be equal and so, there is an implementation overlap.
    #[must_use]
    pub fn implemented_types_can_be_equal(
        &self,
        left_type_parameters: &[IdentifierAST],
        left_implemented_type: Type,
        right_type_parameters: &[IdentifierAST],
        right_implemented_type: Type,
    ) -> bool {
        match (left_implemented_type, right_implemented_type) {
            (Type::Unit, Type::Unit) => true,

            // (_, _) overlaps with (int32, (String, _))
            (
                Type::Tuple {
                    element_types: left,
                },
                Type::Tuple {
                    element_types: right,
                },
            ) => zip(left, right).all(|(left, right)| {
                self.implemented_types_can_be_equal(
                    left_type_parameters,
                    left,
                    right_type_parameters,
                    right,
                )
            }),
            // `(_, int32): _` overlaps with `(_, _): int32`
            // E.g. `(int32, int32): int32` can appear in both implementations
            (
                Type::Function {
                    parameter_types: left_parameter_types,
                    return_type: left_return_type,
                },
                Type::Function {
                    parameter_types: right_parameter_types,
                    return_type: right_return_type,
                },
            ) => {
                zip(left_parameter_types, right_parameter_types).all(|(left, right)| {
                    self.implemented_types_can_be_equal(
                        left_type_parameters,
                        left,
                        right_type_parameters,
                        right,
                    )
                }) && self.implemented_types_can_be_equal(
                    left_type_parameters,
                    *left_return_type,
                    right_type_parameters,
                    *right_return_type,
                )
            }
            // dyn Debug + ToString overlaps with dyn ToString
            (
                Type::TraitObject {
                    bounds: left_bounds,
                },
                Type::TraitObject {
                    bounds: right_bounds,
                },
            ) => left_bounds.iter().any(|b| right_bounds.contains(b)),
            (Type::Variable(left_var), Type::Variable(right_var)) => left_var == right_var,

            // `_` overlaps with `a`
            // `a` overlaps with `_`
            // `_` overlaps with `_`
            // `a` overlaps with `a`
            (Type::Path { path: left_path }, Type::Path { path: right_path }) => {
                return left_path == right_path
                    || left_type_parameters.iter().any(|parameter| {
                        left_path.check_single_identifier_type_constructor(parameter.symbol)
                    })
                    || right_type_parameters.iter().any(|parameter| {
                        right_path.check_single_identifier_type_constructor(parameter.symbol)
                    })
            }

            // _ overlaps with any type
            (Type::Path { path }, _) => left_type_parameters
                .iter()
                .any(|parameter| path.check_single_identifier_type_constructor(parameter.symbol)),
            (_, Type::Path { path }) => right_type_parameters
                .iter()
                .any(|parameter| path.check_single_identifier_type_constructor(parameter.symbol)),

            _ => false,
        }
    }

    pub fn check_overlap_with_candidate(
        &self,
        trait_path: &Path,
        candidate_implementation_data: &ImplementationData,
        implementation_data: &ImplementationData,
    ) {
        todo!()
    }
}

#[derive(Debug, Clone, Hash)]
pub struct TraitData {
    pub definition_id: DefinitionID,
    pub generic_parameters: Vec<GenericParameterData>,
    pub constraints: Vec<ConstraintData>,
    pub implementations: Vec<ImplementationData>,
}

#[derive(Debug, Clone, Hash)]
pub struct ImplementationData {
    pub definition_id: DefinitionID,
    pub generic_parameters: Vec<GenericParameterData>,
    pub constraints: Vec<ConstraintData>,
    pub ty: Type,
    pub implemented_type_location: Location,
}

#[derive(Debug, Clone, Hash)]
pub enum ConstraintData {
    Satisfies { left: Type, right: Bounds },
    Eq { left: Type, right: Type },
}

#[derive(Debug, Clone, Hash)]
pub struct GenericParameterData {
    pub identifier: IdentifierAST,
    pub default_value: Type,
}
