#![allow(warnings)]

use std::{
    ops::{ControlFlow, Deref},
    rc::{self, Rc},
    sync::Arc,
};

use derive_more::Display;
use diagnostics::ExpectedInterface;
use hir_storage::HIRStorage;
use parking_lot::RwLock;
use ry_ast::{IdentifierAST, ImportPath, Visibility};
use ry_diagnostics::{BuildDiagnostic, Diagnostics};
use ry_filesystem::location::Location;
use ry_fx_hash::{FxHashMap, FxHashSet};
use ry_hir::{Module, TypeAlias};
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
use thir_storage::THIRStorage;
use type_variable_factory::TypeVariableFactory;

use crate::diagnostics::{BoundsInTypeAliasDiagnostic, ExpectedType};

pub mod diagnostics;
pub mod hir_storage;
pub mod thir_storage;
pub mod type_variable_factory;

#[derive(Debug)]
pub enum ModuleItemState {
    HIR(ry_hir::ModuleItem),
    THIR(ry_thir::ModuleItem),
}

impl ModuleItemState {
    #[inline]
    #[must_use]
    pub const fn hir(&self) -> Option<&ry_hir::ModuleItem> {
        match self {
            Self::HIR(hir) => Some(hir),
            _ => None,
        }
    }

    #[inline]
    #[must_use]
    pub fn hir_or_panic(&self) -> &ry_hir::ModuleItem {
        self.hir()
            .unwrap_or_else(|| panic!("expected HIR, found already analyzed THIR"))
    }
}

/// Context for type checking stage of compilation.
#[derive(Debug)]
pub struct TypeCheckingContext<'i, 'p, 'd> {
    /// Global name resolution environment.
    resolution_environment: ResolutionEnvironment,

    /// HIR global storage.
    hir_storage: RwLock<HIRStorage>,

    /// THIR global storage.
    thir_storage: RwLock<THIRStorage>,

    /// List of type aliases, that have been recursivly analyzed. Used to find
    /// type alias cycles.
    type_alias_stack: FxHashSet<DefinitionID>,

    /// Identifier interner.
    identifier_interner: &'i IdentifierInterner,

    /// Path interner.
    path_interner: &'p PathInterner,

    /// Used to produce new type variables.
    type_variable_factory: TypeVariableFactory,

    /// Storage of signatures for module items.
    signatures: FxHashMap<DefinitionID, Arc<ModuleItemSignature>>,
    substitutions: FxHashMap<IdentifierID, Type>,

    /// Diagnostics.
    diagnostics: &'d RwLock<Diagnostics>,
}

impl<'i, 'p, 'd> TypeCheckingContext<'i, 'p, 'd> {
    /// Creates a new empty type checking context.
    pub fn new(
        path_interner: &'p PathInterner,
        identifier_interner: &'i IdentifierInterner,
        diagnostics: &'d RwLock<Diagnostics>,
    ) -> Self {
        Self {
            path_interner,
            identifier_interner,
            diagnostics,
            type_alias_stack: FxHashSet::default(),
            hir_storage: RwLock::new(HIRStorage::new()),
            thir_storage: RwLock::new(THIRStorage::new()),
            resolution_environment: ResolutionEnvironment::new(),
            type_variable_factory: TypeVariableFactory::new(),
            substitutions: FxHashMap::default(),
            signatures: FxHashMap::default(),
        }
    }

    /// Adds a not analyzed module HIR into the context.
    pub fn add_module_hir(&mut self, module_id: ModuleID, path: Path, hir: Module) {
        let mut imports = FxHashMap::default();
        let mut enums = FxHashMap::default();

        for item in hir.items {
            self.add_item_hir(module_id, item, &mut imports, &mut enums);
        }

        self.resolution_environment
            .module_paths
            .insert(module_id, path);
    }

    /// Adds a not analyzed module item HIR into the context.
    pub fn add_item_hir(
        &mut self,
        module_id: ModuleID,
        item: ry_hir::ModuleItem,
        imports: &mut FxHashMap<IdentifierID, NameBinding>,
        enums: &mut FxHashMap<DefinitionID, EnumData>,
    ) {
        match item {
            ry_hir::ModuleItem::Import { path, .. } => {
                self.add_import_hir(path, imports);
            }
            ry_hir::ModuleItem::Enum {
                visibility,
                name: IdentifierAST { id: name_id, .. },
                items,
                ..
            } => {
                self.add_enum_hir(module_id, visibility, name_id, items, enums);
            }
            _ => {
                let definition_id = DefinitionID {
                    name_id: item.name().unwrap(),
                    module_id,
                };

                self.resolution_environment
                    .visibilities
                    .insert(definition_id, item.visibility());
                self.hir_storage
                    .write()
                    .add_module_item(definition_id, item);
            }
        }
    }

    /// Adds an import into the context (adds it into its inner name resolution context).
    fn add_import_hir(
        &self,
        path: ry_hir::ImportPath,
        imports: &mut FxHashMap<IdentifierID, NameBinding>,
    ) {
        let ImportPath { path, r#as } = path;

        let name_id = if let Some(r#as) = r#as {
            r#as
        } else {
            *path.identifiers.last().unwrap()
        }
        .id;

        let Some(binding) = self.resolution_environment.resolve_path(
            path.clone(),
            self.identifier_interner,
            self.diagnostics,
        ) else {
            return;
        };

        imports.insert(name_id, binding);
    }

    /// Adds a not yet analyzed enum module item HIR into the context.
    fn add_enum_hir(
        &mut self,
        module_id: ModuleID,
        visibility: Visibility,
        name_id: IdentifierID,
        items: Vec<ry_hir::EnumItem>,
        enums: &mut FxHashMap<DefinitionID, EnumData>,
    ) {
        let definition_id = DefinitionID { name_id, module_id };

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
        enums.insert(definition_id, EnumData { items: items_data });
    }

    /// Resolves all imports in the name resolution context.
    ///
    /// **WARNING**: The function must be called before any actions related to analysis or
    /// name resolution, because if not it will cause panics when trying to work with
    /// module imports.
    #[inline]
    pub fn process_imports(&mut self) {
        self.resolution_environment
            .resolve_imports(self.identifier_interner, self.diagnostics);
    }

    /// Converts a type representation from HIR into [`Type`].
    pub fn resolve_type(
        &self,
        ty: &ry_hir::Type,
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
                    return_type,
                    generic_parameter_scope,
                    module_scope,
                )?),
            }),
            ry_hir::Type::InterfaceObject { location, bounds } => self
                .resolve_bounds(&bounds, module_scope)
                .map(|bounds| Type::InterfaceObject { bounds }),
        }
    }

    /// Converts a type constructor from HIR into [`TypeConstructor`].
    fn resolve_type_constructor(
        &self,
        ty: &ry_hir::TypeConstructor,
        generic_parameter_scope: Option<&GenericParameterScope>,
        module_scope: &ModuleScope,
    ) -> Option<TypeConstructor> {
        if let Some(generic_parameter_scope) = generic_parameter_scope {
            let mut identifiers_iter = ty.path.identifiers.iter();
            let possible_generic_parameter_name = identifiers_iter.next().unwrap();

            if identifiers_iter.next().is_none() && ty.arguments.is_empty() {
                if generic_parameter_scope.contains(possible_generic_parameter_name.id) {
                    return Some(TypeConstructor {
                        path: Path {
                            identifiers: vec![possible_generic_parameter_name.id],
                        },
                        arguments: vec![],
                    });
                }
            }
        }

        let Some(name_binding) = module_scope.resolve_path(
            ty.path.clone(),
            self.identifier_interner,
            self.diagnostics,
            &self.resolution_environment,
        ) else {
            return None;
        };

        let name_binding_kind = name_binding.kind();

        if !name_binding_kind.is_module_item() {
            self.diagnostics.write().add_single_file_diagnostic(
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

    /// Resolves type arguments.
    fn resolve_type_arguments(
        &self,
        hir: &[ry_hir::Type],
        generic_parameter_scope: Option<&GenericParameterScope>,
        module_scope: &ModuleScope,
    ) -> Option<Vec<Type>> {
        hir.into_iter()
            .map(|ty| self.resolve_type(ty, generic_parameter_scope, module_scope))
            .collect::<Option<_>>()
    }

    fn unwrap_type_alias(&self, path: Path) -> Type {
        let definition_id = self.resolve_type_signature_by_path(path);
        todo!()
    }

    fn implements(&self, ty: Type, interface: TypeConstructor) -> bool {
        match ty {
            Type::Constructor(constructor) => {
                let signature = self.resolve_type_signature_by_path(constructor.path);

                match signature.as_ref() {
                    ModuleItemSignature::TypeAlias(alias) => {}
                    _ => {}
                }

                todo!()
            }
            _ => false, // implement builtin interfaces later
        }
    }

    fn resolve_interface(
        &self,
        interface: ry_hir::TypeConstructor,
        generic_parameter_scope: Option<&GenericParameterScope>,
        module_scope: &ModuleScope,
    ) -> Option<TypeConstructor> {
        let Some(name_binding) = module_scope.resolve_path(
            interface.path.clone(),
            self.identifier_interner,
            self.diagnostics,
            &self.resolution_environment,
        ) else {
            return None;
        };

        let signature = self.resolve_signature(name_binding, module_scope)?;

        match signature.as_ref() {
            ModuleItemSignature::Interface(_) => Some(TypeConstructor {
                path: Path {
                    identifiers: interface
                        .path
                        .identifiers
                        .iter()
                        .map(|identifier| identifier.id)
                        .collect(),
                },
                arguments: self.resolve_type_arguments(
                    &interface.arguments,
                    generic_parameter_scope,
                    module_scope,
                )?,
            }),
            _ => unreachable!(),
        }
    }

    fn resolve_bounds(
        &self,
        bounds: &[ry_hir::TypeConstructor],
        module_scope: &ModuleScope,
    ) -> Option<Vec<TypeConstructor>> {
        bounds
            .into_iter()
            .map(|bound| self.resolve_interface(bound.clone(), None, module_scope))
            .collect::<Option<_>>()
    }

    fn resolve_type_signature_by_definition_id(
        &self,
        definition_id: DefinitionID,
    ) -> Arc<ModuleItemSignature> {
        todo!()
    }

    fn resolve_type_signature_by_path(&self, path: Path) -> Arc<ModuleItemSignature> {
        todo!()
    }

    fn resolve_interface_signature_by_definition_id(
        &self,
        definition_id: DefinitionID,
    ) -> Arc<ModuleItemSignature> {
        todo!()
    }

    fn resolve_interface_signature_by_path(&self, path: Path) -> Arc<ModuleItemSignature> {
        todo!()
    }

    fn resolve_signature(
        &self,
        name_binding: NameBinding,
        module_scope: &ModuleScope,
    ) -> Option<Arc<ModuleItemSignature>> {
        match name_binding {
            NameBinding::Enum(definition_id)
            | NameBinding::Interface(definition_id)
            | NameBinding::Function(definition_id)
            | NameBinding::TypeAlias(definition_id)
            | NameBinding::Struct(definition_id) => {
                if let Some(signature) = self.signatures.get(&definition_id).cloned() {
                    Some(signature)
                } else {
                    self.analyze_signature(name_binding, module_scope)
                }
            }
            _ => unreachable!(),
        }
    }

    fn analyze_signature(
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

    fn analyze_type_alias_signature(
        &self,
        definition_id: DefinitionID,
        module_scope: &ModuleScope,
    ) -> Option<Arc<ModuleItemSignature>> {
        let hir_storage_reader = self.hir_storage.read();
        let alias = hir_storage_reader.resolve_type_alias_hir_or_panic(definition_id);

        let (generic_parameter_scope, _) = self.analyze_generic_parameters_and_bounds(
            true,
            alias.name.location,
            None,
            &alias.generic_parameters,
            &[],
            module_scope,
        )?;
        let generic_parameter_scope = Some(&generic_parameter_scope);

        let value = self.resolve_type(&alias.value, generic_parameter_scope, module_scope);

        todo!()
    }

    fn analyze_generic_parameters_and_bounds<'gs>(
        &'gs self,
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
            if let Some(data) = generic_parameter_scope.resolve(parameter_hir.name.id) {}

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
                        BoundsInTypeAliasDiagnostic {
                            alias_name_location: module_item_name_location,
                            bounds_location: Location {
                                start: bounds_hir.first().unwrap().location.start,
                                end: bounds_hir.last().unwrap().location.end,
                                file_path_id: module_item_name_location.file_path_id,
                            },
                        }
                        .build(),
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
