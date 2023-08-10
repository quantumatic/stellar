#![allow(warnings)]

use generic_parameter_scope::GenericParameterScope;
use ry_ast::{DefinitionID, ImportPath};
use ry_diagnostics::GlobalDiagnostics;
use ry_fx_hash::FxHashMap;
use ry_hir::Module;
use ry_interner::{IdentifierInterner, PathID, PathInterner, Symbol};
use ry_name_resolution::{
    EnumData, EnumItemID, ModuleScope, NameBinding, Path, ResolutionEnvironment,
};
use ry_thir::{
    ty::{self, Type, TypeConstructor},
    InterfaceSignature, ModuleItemSignature,
};
use type_variable_factory::TypeVariableFactory;

pub mod diagnostics;
pub mod generic_parameter_scope;
pub mod type_variable_factory;

#[derive(Debug)]
pub struct TypeCheckingContext<'i, 'p, 'd> {
    identifier_interner: &'i mut IdentifierInterner,
    path_interner: &'p PathInterner,
    resolution_environment: ResolutionEnvironment,
    type_variable_factory: TypeVariableFactory,
    items: FxHashMap<DefinitionID, ModuleItem>,
    signatures: FxHashMap<DefinitionID, ModuleItemSignature>,
    substitutions: FxHashMap<Symbol, Type>,
    diagnostics: &'d mut GlobalDiagnostics,
}

#[derive(Debug)]
pub enum ModuleItem {
    HIR(ry_hir::ModuleItem),
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

    pub fn add_package(&mut self, name: Symbol, package_root_module_path_id: PathID) {
        self.resolution_environment
            .packages_root_modules
            .insert(name, package_root_module_path_id);
    }

    pub fn add_module(&mut self, module_path_id: PathID, path: Path, hir: Module) {
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
                        symbol: name.symbol,
                        module_path_id,
                    };

                    let mut items_data = FxHashMap::default();

                    for item in items {
                        items_data.insert(
                            item.symbol(),
                            EnumItemID {
                                enum_definition_id: definition_id,
                                item_name: item.symbol(),
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
                        module_path_id,
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
            .insert(module_path_id, path);
    }

    #[inline]
    pub fn resolve_imports(&mut self) {
        self.resolution_environment
            .resolve_imports(self.identifier_interner, self.diagnostics);
    }

    pub fn lower_type(
        &mut self,
        ty: ry_hir::Type,
        identifier_interner: &mut IdentifierInterner,
        diagnostics: &mut GlobalDiagnostics,
        generic_parameter_scope: &GenericParameterScope,
        module_scope: &ModuleScope,
        environment: &ResolutionEnvironment,
    ) -> Type {
        match ty {
            ry_hir::Type::Constructor(constructor) => self.resolve_type_constructor(
                constructor,
                identifier_interner,
                diagnostics,
                generic_parameter_scope,
                module_scope,
                environment,
            ),
            ry_hir::Type::Tuple { element_types, .. } => Type::Tuple {
                element_types: element_types
                    .into_iter()
                    .map(|element| {
                        self.lower_type(
                            element,
                            identifier_interner,
                            diagnostics,
                            generic_parameter_scope,
                            module_scope,
                            environment,
                        )
                    })
                    .collect(),
            },
            ry_hir::Type::Function {
                parameter_types,
                return_type,
                ..
            } => Type::Function {
                parameter_types: parameter_types
                    .into_iter()
                    .map(|parameter| {
                        self.lower_type(
                            parameter,
                            identifier_interner,
                            diagnostics,
                            generic_parameter_scope,
                            module_scope,
                            environment,
                        )
                    })
                    .collect(),
                return_type: Box::new(self.lower_type(
                    *return_type,
                    identifier_interner,
                    diagnostics,
                    generic_parameter_scope,
                    module_scope,
                    environment,
                )),
            },
            ry_hir::Type::InterfaceObject { location, bounds } => Type::InterfaceObject {
                bounds: bounds
                    .into_iter()
                    .map(|interface| self.resolve_interface_type_constructor(interface))
                    .collect(),
            },
        }
    }

    fn resolve_type_constructor(
        &mut self,
        ty: ry_ast::TypeConstructor,
        identifier_interner: &mut IdentifierInterner,
        diagnostics: &mut GlobalDiagnostics,
        generic_parameter_scope: &GenericParameterScope,
        module_scope: &ModuleScope,
        environment: &ResolutionEnvironment,
    ) -> Type {
        let Some(name_binding) =
            module_scope.resolve_path(ty.path, identifier_interner, diagnostics, environment)
        else {
            return self
                .type_variable_factory
                .make_unknown_type_placeholder(ty.location);
        };

        todo!()
    }

    fn resolve_interface_type_constructor(
        &self,
        interface: ry_ast::TypeConstructor,
    ) -> TypeConstructor {
        todo!()
    }
}
