#![allow(warnings)]

use std::{ops::ControlFlow, sync::Arc};

use derive_more::Display;
use diagnostics::ExpectedInterface;
use ry_ast::{IdentifierAST, ImportPath, Visibility};
use ry_diagnostics::{BuildDiagnostic, GlobalDiagnostics};
use ry_filesystem::location::Location;
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

use crate::diagnostics::{BoundsInTypeAliasDiagnostic, ExpectedType};

pub mod diagnostics;
pub mod type_variable_factory;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum BindingKind {
    #[display(fmt = "package")]
    Package,

    #[display(fmt = "module")]
    Module,

    #[display(fmt = "type")]
    Type,

    #[display(fmt = "interface")]
    Interface,

    #[display(fmt = "function")]
    Function,

    #[display(fmt = "enum item")]
    EnumItem,
}

impl From<NameBindingKind> for BindingKind {
    #[inline]
    fn from(value: NameBindingKind) -> Self {
        match value {
            NameBindingKind::Package => Self::Package,
            NameBindingKind::Module => Self::Module,
            NameBindingKind::EnumItem => Self::EnumItem,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct TypeCheckingContext<'p, 'g> {
    pub resolution_environment: ResolutionEnvironment,

    path_interner: &'p PathInterner,
    type_variable_factory: TypeVariableFactory,
    signatures: FxHashMap<DefinitionID, Arc<ModuleItemSignature<'g>>>,
    substitutions: FxHashMap<IdentifierID, Type>,
}

#[derive(Default, Debug)]
pub struct HIRStorage {
    items: FxHashMap<DefinitionID, ry_hir::ModuleItem>,
}

impl HIRStorage {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn resolve(&self, definition_id: DefinitionID) -> Option<&ry_hir::ModuleItem> {
        self.items.get(&definition_id)
    }

    #[inline]
    #[must_use]
    pub fn resolve_or_panic(&self, definition_id: DefinitionID) -> &ry_hir::ModuleItem {
        self.resolve(definition_id)
            .unwrap_or_else(|| panic!("Expected definition {:?} to exist", definition_id))
    }
}

#[derive(Default, Debug)]
pub struct THIRStorage<'g> {
    items: FxHashMap<DefinitionID, ry_thir::ModuleItem<'g>>,
}

impl<'g> THIRStorage<'g> {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn resolve(&self, definition_id: DefinitionID) -> Option<&ry_thir::ModuleItem<'g>> {
        self.items.get(&definition_id)
    }

    #[inline]
    #[must_use]
    pub fn resolve_or_panic(&self, definition_id: DefinitionID) -> &ry_thir::ModuleItem<'g> {
        self.resolve(definition_id)
            .unwrap_or_else(|| panic!("Expected definition {:?} to exist", definition_id))
    }
}

impl<'p, 'g> TypeCheckingContext<'p, 'g> {
    pub fn new(path_interner: &'p PathInterner) -> Self {
        Self {
            path_interner,
            resolution_environment: ResolutionEnvironment::new(),
            type_variable_factory: TypeVariableFactory::new(),
            substitutions: FxHashMap::default(),
            signatures: FxHashMap::default(),
        }
    }

    pub fn add_module_hir(
        &mut self,
        module_id: ModuleID,
        path: Path,
        hir: Module,
        hir_storage: &mut HIRStorage,
        identifier_interner: &mut IdentifierInterner,
        diagnostics: &mut GlobalDiagnostics,
    ) {
        let mut imports = FxHashMap::default();
        let mut enums = FxHashMap::default();

        for (idx, item) in hir.items.into_iter().enumerate() {
            self.add_item_hir(
                module_id,
                item,
                &mut imports,
                &mut enums,
                hir_storage,
                identifier_interner,
                diagnostics,
            );
        }

        self.resolution_environment
            .module_paths
            .insert(module_id, path);
    }

    pub fn add_item_hir(
        &mut self,
        module_id: ModuleID,
        item: ry_hir::ModuleItem,
        imports: &mut FxHashMap<IdentifierID, NameBinding>,
        enums: &mut FxHashMap<DefinitionID, EnumData>,
        hir_storage: &mut HIRStorage,
        identifier_interner: &mut IdentifierInterner,
        diagnostics: &mut GlobalDiagnostics,
    ) {
        match item {
            ry_hir::ModuleItem::Import { path, .. } => {
                self.add_import_hir(path, imports, identifier_interner, diagnostics);
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
                hir_storage.items.insert(definition_id, item);
            }
        }
    }

    fn add_import_hir(
        &mut self,
        path: ry_hir::ImportPath,
        imports: &mut FxHashMap<IdentifierID, NameBinding>,
        identifier_interner: &mut IdentifierInterner,
        diagnostics: &mut GlobalDiagnostics,
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
            identifier_interner,
            diagnostics,
        ) else {
            return;
        };

        imports.insert(name_id, binding);
    }

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

    #[inline]
    pub fn process_imports(
        &mut self,
        identifier_interner: &mut IdentifierInterner,
        diagnostics: &mut GlobalDiagnostics,
    ) {
        self.resolution_environment
            .resolve_imports(identifier_interner, diagnostics);
    }

    pub fn resolve_type(
        &mut self,
        ty: &ry_hir::Type,
        generic_parameter_scope: Option<&GenericParameterScope>,
        module_scope: &ModuleScope,
        hir_storage: &HIRStorage,
        thir_storage: &mut THIRStorage,
        identifier_interner: &mut IdentifierInterner,
        diagnostics: &mut GlobalDiagnostics,
    ) -> Option<Type> {
        match ty {
            ry_hir::Type::Constructor(constructor) => self
                .resolve_type_constructor(
                    constructor,
                    generic_parameter_scope,
                    module_scope,
                    identifier_interner,
                    diagnostics,
                )
                .map(Type::Constructor),
            ry_hir::Type::Tuple { element_types, .. } => element_types
                .into_iter()
                .map(|element| {
                    self.resolve_type(
                        element,
                        generic_parameter_scope,
                        module_scope,
                        hir_storage,
                        thir_storage,
                        identifier_interner,
                        diagnostics,
                    )
                })
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
                        self.resolve_type(
                            parameter,
                            generic_parameter_scope,
                            module_scope,
                            hir_storage,
                            thir_storage,
                            identifier_interner,
                            diagnostics,
                        )
                    })
                    .collect::<Option<_>>()?,
                return_type: Box::new(self.resolve_type(
                    return_type,
                    generic_parameter_scope,
                    module_scope,
                    hir_storage,
                    thir_storage,
                    identifier_interner,
                    diagnostics,
                )?),
            }),
            ry_hir::Type::InterfaceObject { location, bounds } => self
                .resolve_bounds(
                    &bounds,
                    module_scope,
                    hir_storage,
                    thir_storage,
                    identifier_interner,
                    diagnostics,
                )
                .map(|bounds| Type::InterfaceObject { bounds }),
        }
    }

    fn resolve_type_constructor(
        &mut self,
        ty: &ry_ast::TypeConstructor,
        generic_parameter_scope: Option<&GenericParameterScope>,
        module_scope: &ModuleScope,
        identifier_interner: &mut IdentifierInterner,
        diagnostics: &mut GlobalDiagnostics,
    ) -> Option<TypeConstructor> {
        if let Some(generic_parameter_scope) = generic_parameter_scope {
            let mut identifiers_iter = ty.path.identifiers.iter();
            let possible_generic_parameter_name = identifiers_iter.next().unwrap();

            if identifiers_iter.next().is_none() {
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
            identifier_interner,
            diagnostics,
            &self.resolution_environment,
        ) else {
            return None;
        };

        let name_binding_kind = name_binding.kind();

        if name_binding_kind != NameBindingKind::ModuleItem {
            diagnostics.add_single_file_diagnostic(
                ty.location.file_path_id,
                ExpectedType {
                    location: ty.location,
                    type_binding_kind: match name_binding_kind {
                        NameBindingKind::Package => BindingKind::Package,
                        NameBindingKind::Module => BindingKind::Module,
                        NameBindingKind::EnumItem => BindingKind::EnumItem,
                        _ => unreachable!(),
                    },
                }
                .build(),
            );

            return None;
        }

        todo!()
    }

    fn resolve_type_arguments(
        &mut self,
        hir: Option<&[ry_hir::Type]>,
        generic_parameter_scope: Option<&GenericParameterScope>,
        module_scope: &ModuleScope,
        hir_storage: &HIRStorage,
        thir_storage: &mut THIRStorage,
        identifier_interner: &mut IdentifierInterner,
        diagnostics: &mut GlobalDiagnostics,
    ) -> Option<Vec<Type>> {
        if let Some(hir) = hir {
            if hir.is_empty() {
                todo!("emit some diagnostics");
            }

            hir.into_iter()
                .map(|ty| {
                    self.resolve_type(
                        ty,
                        generic_parameter_scope,
                        module_scope,
                        hir_storage,
                        thir_storage,
                        identifier_interner,
                        diagnostics,
                    )
                })
                .collect::<Option<_>>()
        } else {
            Some(vec![])
        }
    }

    fn resolve_interface(
        &mut self,
        interface: ry_hir::TypeConstructor,
        generic_parameter_scope: Option<&GenericParameterScope>,
        module_scope: &ModuleScope,
        hir_storage: &HIRStorage,
        thir_storage: &mut THIRStorage,
        identifier_interner: &mut IdentifierInterner,
        diagnostics: &mut GlobalDiagnostics,
    ) -> Option<TypeConstructor> {
        let Some(name_binding) = module_scope.resolve_path(
            interface.path.clone(),
            identifier_interner,
            diagnostics,
            &self.resolution_environment,
        ) else {
            return None;
        };

        let definition_id = match name_binding {
            NameBinding::ModuleItem(definition_id) => definition_id,
            _ => {
                diagnostics.add_single_file_diagnostic(
                    interface.location.file_path_id,
                    ExpectedInterface {
                        location: interface.location,
                        type_binding_kind: name_binding.kind().into(),
                    }
                    .build(),
                );

                return None;
            }
        };

        let signature = self.resolve_signature(
            definition_id,
            module_scope,
            hir_storage,
            thir_storage,
            identifier_interner,
            diagnostics,
        )?;

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
                    interface
                        .arguments
                        .as_ref()
                        .map(|arguments| arguments.as_slice()),
                    generic_parameter_scope,
                    module_scope,
                    hir_storage,
                    thir_storage,
                    identifier_interner,
                    diagnostics,
                )?,
            }),
            _ => {
                diagnostics.add_single_file_diagnostic(
                    interface.location.file_path_id,
                    ExpectedInterface {
                        location: interface.location,
                        type_binding_kind: match signature.as_ref() {
                            ModuleItemSignature::Type(_) | ModuleItemSignature::TypeAlias(_) => {
                                BindingKind::Type
                            }
                            ModuleItemSignature::Function(_) => BindingKind::Function,
                            _ => unreachable!(),
                        },
                    }
                    .build(),
                );

                None
            }
        }
    }

    fn resolve_bounds(
        &mut self,
        bounds: &ry_hir::Bounds,
        module_scope: &ModuleScope,
        hir_storage: &HIRStorage,
        thir_storage: &mut THIRStorage,
        identifier_interner: &mut IdentifierInterner,
        diagnostics: &mut GlobalDiagnostics,
    ) -> Option<Vec<TypeConstructor>> {
        bounds
            .into_iter()
            .map(|bound| {
                self.resolve_interface(
                    bound.clone(),
                    None,
                    module_scope,
                    hir_storage,
                    thir_storage,
                    identifier_interner,
                    diagnostics,
                )
            })
            .collect::<Option<_>>()
    }

    fn resolve_signature(
        &mut self,
        definition_id: DefinitionID,
        module_scope: &ModuleScope,
        hir_storage: &HIRStorage,
        thir_storage: &mut THIRStorage,
        identifier_interner: &mut IdentifierInterner,
        diagnostics: &mut GlobalDiagnostics,
    ) -> Option<Arc<ModuleItemSignature>> {
        if let Some(signature) = self.signatures.get(&definition_id).cloned() {
            return Some(signature);
        }

        self.analyze_signature(
            definition_id,
            module_scope,
            hir_storage,
            thir_storage,
            identifier_interner,
            diagnostics,
        )
    }

    fn analyze_signature(
        &mut self,
        definition_id: DefinitionID,
        module_scope: &ModuleScope,
        hir_storage: &HIRStorage,
        thir_storage: &mut THIRStorage,
        identifier_interner: &mut IdentifierInterner,
        diagnostics: &mut GlobalDiagnostics,
    ) -> Option<Arc<ModuleItemSignature>> {
        match hir_storage.resolve_or_panic(definition_id) {
            ry_hir::ModuleItem::TypeAlias(alias) => self.analyze_type_alias_signature(
                alias,
                module_scope,
                hir_storage,
                thir_storage,
                identifier_interner,
                diagnostics,
            ),
            _ => todo!(),
        }
    }

    fn analyze_type_alias_signature(
        &mut self,
        hir: &ry_hir::TypeAlias,
        module_scope: &ModuleScope,
        hir_storage: &HIRStorage,
        thir_storage: &mut THIRStorage,
        identifier_interner: &mut IdentifierInterner,
        diagnostics: &mut GlobalDiagnostics,
    ) -> Option<Arc<ModuleItemSignature>> {
        let (generic_parameter_scope, _) = self.analyze_generic_parameters_and_bounds(
            true,
            hir.name.location,
            None,
            &hir.generic_parameters,
            &[],
            module_scope,
            hir_storage,
            thir_storage,
            identifier_interner,
            diagnostics,
        )?;
        let generic_parameter_scope = Some(&generic_parameter_scope);

        todo!()
    }

    fn analyze_generic_parameters_and_bounds<'gs>(
        &'gs mut self,
        is_type_alias: bool,
        module_item_name_location: Location,
        parent_generic_parameter_scope: Option<&'gs GenericParameterScope>,
        generic_parameters_hir: &[ry_hir::GenericParameter],
        where_predicates_hir: &[ry_hir::WherePredicate],
        module_scope: &ModuleScope,
        hir_storage: &HIRStorage,
        thir_storage: &mut THIRStorage,
        identifier_interner: &mut IdentifierInterner,
        diagnostics: &mut GlobalDiagnostics,
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
                    hir_storage,
                    thir_storage,
                    identifier_interner,
                    diagnostics,
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
                    diagnostics.add_single_file_diagnostic(
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
                        hir_storage,
                        thir_storage,
                        identifier_interner,
                        diagnostics,
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
                &where_predicate_hir.ty,
                Some(&generic_parameter_scope),
                module_scope,
                hir_storage,
                thir_storage,
                identifier_interner,
                diagnostics,
            )?;
        }

        Some((generic_parameter_scope, predicates))
    }
}
