#![allow(warnings)]

use std::sync::Arc;

use ry_ast::ImportPath;
use ry_diagnostics::{BuildDiagnostic, GlobalDiagnostics};
use ry_fx_hash::FxHashMap;
use ry_hir::Module;
use ry_interner::{IdentifierID, IdentifierInterner, PathID, PathInterner};
use ry_name_resolution::{
    DefinitionID, EnumData, EnumItemID, ModuleID, ModuleScope, NameBinding, NameBindingKind, Path,
    ResolutionEnvironment,
};
use ry_stable_likely::unlikely;
use ry_thir::{
    generic_parameter_scope::{GenericParameterData, GenericParameterScope},
    ty::{self, Type, TypeConstructor},
    InterfaceSignature, ModuleItemSignature, Predicate, TypeAliasSignature,
};
use type_variable_factory::TypeVariableFactory;

use crate::diagnostics::ExpectedType;

pub mod diagnostics;
pub mod type_variable_factory;

#[derive(Debug)]
pub struct TypeCheckingContext<'i, 'p, 'g, 'd> {
    pub resolution_environment: ResolutionEnvironment,

    identifier_interner: &'i mut IdentifierInterner,
    path_interner: &'p PathInterner,
    type_variable_factory: TypeVariableFactory,
    signatures: FxHashMap<DefinitionID, Arc<ModuleItemSignature<'g>>>,
    substitutions: FxHashMap<IdentifierID, Type>,
    diagnostics: &'d mut GlobalDiagnostics,
}

#[derive(Debug)]
pub enum ModuleItem<'g> {
    HIR(ry_hir::ModuleItem),
    THIR(ry_thir::ModuleItem<'g>),
}

#[derive(Default, Debug)]
pub struct IRStorage<'g> {
    items: FxHashMap<DefinitionID, ModuleItem<'g>>,
}

impl IRStorage<'_> {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<'i, 'p, 'g, 'd> TypeCheckingContext<'i, 'p, 'g, 'd> {
    pub fn new(
        identifier_interner: &'i mut IdentifierInterner,
        path_interner: &'p PathInterner,
        diagnostics: &'d mut GlobalDiagnostics,
    ) -> Self {
        Self {
            identifier_interner,
            path_interner,
            resolution_environment: ResolutionEnvironment::new(),
            type_variable_factory: TypeVariableFactory::new(),
            substitutions: FxHashMap::default(),
            signatures: FxHashMap::default(),
            diagnostics,
        }
    }

    pub fn add_module(
        &mut self,
        module_id: ModuleID,
        path: Path,
        hir: Module,
        ir_storage: &mut IRStorage,
    ) {
        let mut imports = FxHashMap::default();
        let mut enums = FxHashMap::default();

        for (idx, item) in hir.items.into_iter().enumerate() {
            match item {
                ry_hir::ModuleItem::Import {
                    location,
                    path: ImportPath { path, r#as },
                } => {
                    let name = if let Some(r#as) = r#as {
                        r#as
                    } else {
                        *path.identifiers.last().unwrap()
                    };

                    let Some(binding) = self.resolution_environment.resolve_path(
                        path,
                        self.identifier_interner,
                        self.diagnostics,
                    ) else {
                        continue;
                    };

                    imports.insert(name, binding);
                }
                ry_hir::ModuleItem::Enum {
                    visibility,
                    name,
                    items,
                    ..
                } => {
                    let definition_id = DefinitionID {
                        symbol: name.id,
                        module_id,
                    };

                    let mut items_data = FxHashMap::default();

                    for item in items {
                        items_data.insert(
                            item.symbol(),
                            EnumItemID {
                                enum_definition_id: definition_id,
                                item_id: item.symbol(),
                            },
                        );
                    }

                    self.resolution_environment
                        .visibilities
                        .insert(definition_id, visibility);
                    enums.insert(name, EnumData { items: items_data });
                }
                _ => {
                    let definition_id = DefinitionID {
                        symbol: item.name().unwrap(),
                        module_id,
                    };

                    self.resolution_environment
                        .visibilities
                        .insert(definition_id, item.visibility());
                    ir_storage
                        .items
                        .insert(definition_id, ModuleItem::HIR(item));
                }
            }
        }

        self.resolution_environment
            .module_paths
            .insert(module_id, path);
    }

    #[inline]
    pub fn resolve_imports(&mut self) {
        self.resolution_environment
            .resolve_imports(self.identifier_interner, self.diagnostics);
    }

    pub fn resolve_type(
        &mut self,
        ty: ry_hir::Type,
        generic_parameter_scope: Option<&GenericParameterScope>,
        module_scope: &ModuleScope,
    ) -> Option<Type> {
        match ty {
            ry_hir::Type::Constructor(constructor) => self
                .resolve_type_constructor(constructor, generic_parameter_scope, module_scope)
                .map(Type::Constructor),
            ry_hir::Type::Tuple { element_types, .. } => element_types
                .into_iter()
                .map(|element| self.resolve_type(element, generic_parameter_scope, module_scope))
                .collect::<Option<Vec<_>>>()
                .map(|element_types| Type::Tuple { element_types }),
            ry_hir::Type::Function {
                parameter_types,
                return_type,
                ..
            } => Some(Type::Function {
                parameter_types: parameter_types
                    .into_iter()
                    .map(|parameter| {
                        self.resolve_type(parameter, generic_parameter_scope, module_scope)
                    })
                    .collect::<Option<_>>()?,
                return_type: Box::new(self.resolve_type(
                    *return_type,
                    generic_parameter_scope,
                    module_scope,
                )?),
            }),
            ry_hir::Type::InterfaceObject { location, bounds } => self
                .resolve_bounds(&bounds, module_scope)
                .map(|bounds| Type::InterfaceObject { bounds }),
        }
    }

    fn resolve_type_constructor(
        &mut self,
        ty: ry_ast::TypeConstructor,
        generic_parameter_scope: Option<&GenericParameterScope>,
        module_scope: &ModuleScope,
    ) -> Option<TypeConstructor> {
        let Some(name_binding) = module_scope.resolve_path(
            ty.path,
            self.identifier_interner,
            self.diagnostics,
            &self.resolution_environment,
        ) else {
            return None;
        };

        let name_binding_kind = name_binding.kind();

        if name_binding_kind != NameBindingKind::ModuleItem {
            self.diagnostics.add_single_file_diagnostic(
                ty.location.file_path_id,
                ExpectedType {
                    location: ty.location,
                    name_binding_kind,
                }
                .build(),
            );

            return None;
        }

        todo!()
    }

    fn resolve_interface(
        &mut self,
        interface: ry_ast::TypeConstructor,
        generic_parameter_scope: Option<&GenericParameterScope>,
        module_scope: &ModuleScope,
    ) -> Option<TypeConstructor> {
        let Some(name_binding) = module_scope.resolve_path(
            interface.path,
            self.identifier_interner,
            self.diagnostics,
            &self.resolution_environment,
        ) else {
            return None;
        };

        let name_binding_kind = name_binding.kind();

        if name_binding_kind != NameBindingKind::ModuleItem {
            self.diagnostics.add_single_file_diagnostic(
                interface.location.file_path_id,
                ExpectedType {
                    location: interface.location,
                    name_binding_kind,
                }
                .build(),
            );

            return None;
        }

        todo!()
    }

    fn resolve_bounds(
        &mut self,
        bounds: &ry_ast::Bounds,
        module_scope: &ModuleScope,
    ) -> Option<Vec<TypeConstructor>> {
        bounds
            .into_iter()
            .map(|bound| self.resolve_interface(bound.clone(), None, module_scope))
            .collect::<Option<_>>()
    }

    fn resolve_signature(
        &mut self,
        definition_id: DefinitionID,
        ir_storage: &mut IRStorage,
        module_scope: &ModuleScope,
    ) -> Option<Arc<ModuleItemSignature>> {
        if let Some(signature) = self.signatures.get(&definition_id).cloned() {
            return Some(signature);
        }

        let binding = ir_storage.items.get(&definition_id).unwrap();

        match binding {
            ModuleItem::THIR(_) => {
                panic!(
                    "cannot have a THIR item without a signature at:\n{:?}",
                    definition_id
                );
            }
            ModuleItem::HIR(hir) => self.analyze_signature(hir, module_scope),
        }
    }

    fn analyze_signature(
        &mut self,
        hir: &ry_hir::ModuleItem,
        module_scope: &ModuleScope,
    ) -> Option<Arc<ModuleItemSignature>> {
        match hir {
            ry_hir::ModuleItem::TypeAlias(alias) => {
                self.analyze_type_alias_signature(alias, module_scope)
            }
            _ => todo!(),
        }
    }

    fn analyze_type_alias_signature(
        &mut self,
        hir: &ry_hir::TypeAlias,
        module_scope: &ModuleScope,
    ) -> Option<Arc<ModuleItemSignature>> {
        let (generic_parameter_scope, _) = self.analyze_generic_parameters_and_bounds(
            true,
            None,
            &hir.generic_parameters,
            &[],
            module_scope,
        )?;
        let generic_parameter_scope = Some(&generic_parameter_scope);

        todo!()
    }

    fn analyze_generic_parameters_and_bounds<'a>(
        &'a mut self,
        is_type_alias: bool,
        parent_generic_parameter_scope: Option<&'a GenericParameterScope>,
        generic_parameters_hir: &[ry_hir::GenericParameter],
        where_predicates_hir: &[ry_hir::WherePredicate],
        module_scope: &ModuleScope,
    ) -> Option<(GenericParameterScope, Vec<Predicate>)> {
        let mut generic_parameter_scope =
            GenericParameterScope::new(parent_generic_parameter_scope);
        let mut predicates = Vec::new();

        for parameter_hir in generic_parameters_hir {
            if let Some(data) = generic_parameter_scope.resolve(parameter_hir.name.id) {}

            let default_value = if let Some(default_value_hir) = &parameter_hir.default_value {
                if let Some(ty) = self.resolve_type(
                    default_value_hir.clone(),
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
                    todo!("emit diagnostics");

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
                    ty: Type::primitive(parameter_hir.name.id),
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
                where_predicate_hir.ty.clone(),
                Some(&generic_parameter_scope),
                module_scope,
            )?;
        }

        Some((generic_parameter_scope, predicates))
    }
}
