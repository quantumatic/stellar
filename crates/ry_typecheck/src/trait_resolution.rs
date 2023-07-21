use std::iter::zip;

use ry_ast::{IdentifierAST, TypeBounds};
use ry_diagnostics::{BuildDiagnostic, GlobalDiagnostics};
use ry_fx_hash::{FxHashMap, FxHashSet};
use ry_hir::{
    ty::{Path, Type, TypePathSegment},
    WherePredicate,
};

use crate::diagnostics::DuplicateTraitBoundDiagnostic;

#[derive(Debug, Default)]
pub struct TraitResolutionContext {
    traits: FxHashMap<Path, TraitData>,
    type_implementations: Vec<ImplementationData>,
}

impl TraitResolutionContext {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_implementation(
        &mut self,
        trait_path: Path,
        implementation_data: ImplementationData,
    ) {
        todo!()
    }

    pub fn check_overlap(&self, trait_path: &Path, implementation_data: &ImplementationData) {
        let Some(candidates) = self.traits.get(trait_path).map(|r#trait| &r#trait.implementations) else {
            return;
        };

        for candidate_data in candidates {
            self.check_overlap_with_candidate(trait_path, candidate_data, implementation_data);
        }
    }

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
            Type::Constructor { path } => {
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

    pub fn get_not_constrained_type_parameters(
        &self,
        implemented_type: Type,
        generics: &FxHashSet<IdentifierAST>,
    ) -> FxHashSet<IdentifierAST> {
        let constrained = self.get_constrained_type_parameters(implemented_type, generics);

        generics.difference(&constrained).cloned().collect()
    }

    /// The function checks for type equality in implementations. For example
    /// implemented types in `impl[T] T` and `impl[T, M] HashMap[T, M]` can
    /// be equal and so, there is an implementation overlap.
    pub fn implemented_types_can_be_equal(
        &self,
        left_generics: &[IdentifierAST],
        left_type: Type,
        right_type_generics: &[IdentifierAST],
        right_type: Type,
    ) -> bool {
        match (left_type, right_type) {
            (Type::Unit, Type::Unit) => true,
            (Type::Constructor { path: left_path }, Type::Constructor { path: right_path }) => {
                // impl[T] [T as A].B where T: A {}
                //      ^ unconstrained type parameter
                unreachable!()
            }
            (
                Type::WithQualifiedPath {
                    left: left1,
                    right: right1,
                    segments: segments1,
                },
                Type::WithQualifiedPath {
                    left: left2,
                    right: right2,
                    segments: segments2,
                },
            ) => true,
            (
                Type::Tuple {
                    element_types: left,
                },
                Type::Tuple {
                    element_types: right,
                },
            ) => zip(left, right).all(|(left, right)| {
                self.implemented_types_can_be_equal(
                    left_generics,
                    left.clone(),
                    right_type_generics,
                    right.clone(),
                )
            }),
            (
                Type::TraitObject {
                    bounds: left_bounds,
                },
                Type::TraitObject {
                    bounds: right_bounds,
                },
            ) => left_bounds.iter().any(|b| right_bounds.contains(b)),
            (Type::Variable(left_var), Type::Variable(right_var)) => left_var == right_var,
            _ => unreachable!(),
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

#[derive(Debug, Clone)]
pub struct TraitData {
    generics: Vec<GenericData>,
    constraints: FxHashMap<Type, TypeBounds>,
    implementations: Vec<ImplementationData>,
}

#[derive(Debug, Clone)]
pub struct ImplementationData {
    generics: Vec<GenericData>,
    constraints: FxHashMap<Type, TypeBounds>,
    ty: Type,
}

#[derive(Debug, Clone)]
pub struct GenericData {
    identifier: IdentifierAST,
    default_value: Type,
}
