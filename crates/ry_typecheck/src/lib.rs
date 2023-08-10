#![allow(warnings)]

use generic_parameter_scope::GenericParameterScope;
use ry_ast::ImportPath;
use ry_diagnostics::{BuildDiagnostic, GlobalDiagnostics};
use ry_fx_hash::FxHashMap;
use ry_hir::Module;
use ry_interner::{IdentifierID, IdentifierInterner, PathID, PathInterner};
use ry_name_resolution::{
    DefinitionID, EnumData, EnumItemID, ModuleID, ModuleScope, NameBinding, NameBindingKind, Path,
    ResolutionEnvironment,
};
use ry_thir::{
    ty::{self, Type, TypeConstructor},
    InterfaceSignature, ModuleItemSignature,
};
use type_variable_factory::TypeVariableFactory;

use crate::diagnostics::ExpectedType;

pub mod diagnostics;
pub mod generic_parameter_scope;
pub mod type_variable_factory;

#[derive(Debug)]
pub struct TypeCheckingContext<'i, 'p, 'd> {
    pub resolution_environment: ResolutionEnvironment,

    identifier_interner: &'i mut IdentifierInterner,
    path_interner: &'p PathInterner,
    type_variable_factory: TypeVariableFactory,
    items: FxHashMap<DefinitionID, ModuleItem>,
    signatures: FxHashMap<DefinitionID, ModuleItemSignature>,
    substitutions: FxHashMap<IdentifierID, Type>,
    diagnostics: &'d mut GlobalDiagnostics,
}

#[derive(Debug)]
pub enum ModuleItem {
    HIR(ry_hir::ModuleItem),
    THIR(ry_thir::ModuleItem),
}

impl<'i, 'p, 'd> TypeCheckingContext<'i, 'p, 'd> {
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
            items: FxHashMap::default(),
            signatures: FxHashMap::default(),
            diagnostics,
        }
    }

    pub fn add_module(&mut self, module_id: ModuleID, path: Path, hir: Module) {
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
                    self.items.insert(definition_id, ModuleItem::HIR(item));
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
        generic_parameter_scope: &GenericParameterScope,
        module_scope: &ModuleScope,
    ) -> Option<Type> {
        match ty {
            ry_hir::Type::Constructor(constructor) => {
                self.resolve_type_constructor(constructor, generic_parameter_scope, module_scope)
            }
            ry_hir::Type::Tuple { element_types, .. } => Some(Type::Tuple {
                element_types: element_types
                    .into_iter()
                    .filter_map(|element| {
                        self.resolve_type(element, generic_parameter_scope, module_scope)
                    })
                    .collect(),
            }),
            ry_hir::Type::Function {
                parameter_types,
                return_type,
                ..
            } => Some(Type::Function {
                parameter_types: parameter_types
                    .into_iter()
                    .filter_map(|parameter| {
                        self.resolve_type(parameter, generic_parameter_scope, module_scope)
                    })
                    .collect(),
                return_type: Box::new(self.resolve_type(
                    *return_type,
                    generic_parameter_scope,
                    module_scope,
                )?),
            }),
            ry_hir::Type::InterfaceObject { location, bounds } => Some(Type::InterfaceObject {
                bounds: bounds
                    .into_iter()
                    .filter_map(|interface| {
                        self.resolve_interface_type_constructor(interface, module_scope)
                    })
                    .collect(),
            }),
        }
    }

    fn resolve_type_constructor(
        &mut self,
        ty: ry_ast::TypeConstructor,
        generic_parameter_scope: &GenericParameterScope,
        module_scope: &ModuleScope,
    ) -> Option<Type> {
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

    fn resolve_interface_type_constructor(
        &mut self,
        interface: ry_ast::TypeConstructor,
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

    fn get_signature(&mut self, definition_id: DefinitionID) -> Option<&ModuleItemSignature> {
        if let Some(signature) = self.signatures.get(&definition_id) {
            Some(signature)
        } else {
            let binding = self.items.get(&definition_id).unwrap();

            match binding {
                ModuleItem::THIR(_) => {
                    panic!(
                        "cannot have a THIR item without a signature at:\n{:?}",
                        definition_id
                    );
                }
                ModuleItem::HIR(hir) => todo!(),
            }
        }
    }

    fn analyze_signature<'a>(
        &'a mut self,
        hir: &ry_hir::ModuleItem,
    ) -> Option<&'a ModuleItemSignature> {
        todo!()
    }
}
