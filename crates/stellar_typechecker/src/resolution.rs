use std::sync::Arc;

use stellar_ast::{IdentifierAST, ImportPath, ModuleItemKind, Visibility};
use stellar_fx_hash::FxHashMap;
use stellar_interner::IdentifierID;
use stellar_name_resolution::{
    DefinitionID, EnumData, EnumItemID, ModuleID, ModuleScope, NameBinding, Path,
    ResolvedImportsInModule,
};
use stellar_thir::{
    generic_parameter_scope::GenericParameterScope,
    ty::{Type, TypeConstructor},
    ModuleItemSignature, TypeAliasSignature,
};

use crate::diagnostics::UnderscoreTypeInSignature;
use crate::{
    diagnostics::{ExpectedType, TypeAliasCycleFound},
    TypeCheckingContext,
};

impl TypeCheckingContext<'_, '_, '_> {
    fn add_module(
        &mut self,
        id: ModuleID,
        module_name_id: IdentifierID,
        path: Path,
        hir: stellar_hir::Module,
    ) {
        let mut imports = FxHashMap::default();
        let mut enums = FxHashMap::default();
        let mut bindings = FxHashMap::default();

        for item in hir.items {
            self.add_module_item_hir(id, item, &mut imports, &mut enums, &mut bindings);
        }

        self.name_resolver.add_module_scope(
            id,
            ModuleScope {
                name: module_name_id,
                id,
                path,
                bindings,
                enums,
                imports,
            },
        );
    }

    /// Adds a not analyzed module item HIR into the context.
    fn add_module_item_hir(
        &mut self,
        module_id: ModuleID,
        item: stellar_hir::ModuleItem,
        imports: &mut FxHashMap<IdentifierID, stellar_ast::Path>,
        enums: &mut FxHashMap<IdentifierID, EnumData>,
        bindings: &mut FxHashMap<IdentifierID, NameBinding>,
    ) {
        match item {
            stellar_hir::ModuleItem::Import { path, .. } => {
                self.add_import_hir(path, imports);
            }
            stellar_hir::ModuleItem::Enum(stellar_hir::Enum {
                visibility,
                name: IdentifierAST { id: name_id, .. },
                items,
                ..
            }) => {
                bindings.insert(
                    name_id,
                    NameBinding::Enum(DefinitionID { name_id, module_id }),
                );

                self.add_enum_hir(module_id, visibility, name_id, items, enums);
            }
            _ => {
                let definition_id = DefinitionID {
                    name_id: item.name_or_panic(),
                    module_id,
                };

                self.name_resolver
                    .add_visibility(definition_id, item.visibility());

                bindings.insert(
                    item.name().unwrap(),
                    match item.kind() {
                        ModuleItemKind::Function => NameBinding::Function(definition_id),
                        ModuleItemKind::Struct => NameBinding::Struct(definition_id),
                        ModuleItemKind::TupleLikeStruct => {
                            NameBinding::TupleLikeStruct(definition_id)
                        }
                        ModuleItemKind::Interface => NameBinding::Interface(definition_id),
                        ModuleItemKind::TypeAlias => NameBinding::TypeAlias(definition_id),
                        ModuleItemKind::Enum | ModuleItemKind::Import => unreachable!(),
                    },
                );

                self.hir_storage
                    .write()
                    .add_module_item(definition_id, item);
            }
        }
    }

    /// Adds an import into the context (adds it into its inner name resolution context).
    fn add_import_hir(
        &self,
        path: stellar_hir::ImportPath,
        imports: &mut FxHashMap<IdentifierID, stellar_ast::Path>,
    ) {
        let ImportPath { path, r#as } = path;

        let name_id = if let Some(r#as) = r#as {
            r#as
        } else {
            *path.identifiers.last().unwrap()
        }
        .id;

        imports.insert(name_id, path);
    }

    /// Adds a not yet analyzed enum module item HIR into the context.
    fn add_enum_hir(
        &mut self,
        module_id: ModuleID,
        visibility: Visibility,
        name_id: IdentifierID,
        items: Vec<stellar_hir::EnumItem>,
        enums: &mut FxHashMap<IdentifierID, EnumData>,
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

        self.name_resolver.add_visibility(definition_id, visibility);
        enums.insert(name_id, EnumData { items: items_data });
    }

    /// Resolves all imports in the name resolution context.
    ///
    /// **WARNING**: The function must be called before any actions related to analysis or
    /// name resolution, because if not it will cause panics when trying to work with
    /// module imports.
    #[inline(always)]
    pub fn process_imports(&mut self) {
        self.name_resolver
            .resolve_imports(self.identifier_interner, self.diagnostics);
    }

    /// Converts a type representation from HIR into [`Type`].
    pub fn resolve_type(
        &self,
        ty: &stellar_hir::Type,
        signature: bool,
        generic_parameter_scope: &GenericParameterScope,
        module_scope: &ModuleScope,
    ) -> Option<Type> {
        match ty {
            stellar_hir::Type::Constructor(constructor) => self
                .resolve_type_constructor(constructor, generic_parameter_scope, module_scope)
                .map(Type::Constructor),
            stellar_hir::Type::Tuple { element_types, .. } => element_types
                .iter()
                .map(|element| {
                    self.resolve_type(element, signature, generic_parameter_scope, module_scope)
                })
                .collect::<Option<Vec<_>>>()
                .map(|element_types| Type::Tuple { element_types }),
            stellar_hir::Type::Underscore { location } => {
                if signature {
                    self.diagnostics.write().add_single_file_diagnostic(
                        location.file_path_id,
                        UnderscoreTypeInSignature::new(*location),
                    );

                    None
                } else {
                    todo!()
                }
            }
            stellar_hir::Type::Function {
                parameter_types,
                return_type,
                ..
            } => Some(Type::Function {
                parameter_types: parameter_types
                    .iter()
                    .map(|parameter| {
                        self.resolve_type(
                            parameter,
                            signature,
                            generic_parameter_scope,
                            module_scope,
                        )
                    })
                    .collect::<Option<_>>()?,
                return_type: Box::new(self.resolve_type(
                    return_type,
                    signature,
                    generic_parameter_scope,
                    module_scope,
                )?),
            }),
            stellar_hir::Type::InterfaceObject { bounds, .. } => {
                let bounds =
                    self.resolve_bounds(generic_parameter_scope, signature, bounds, module_scope);

                if bounds.is_empty() {
                    None
                } else {
                    Some(Type::InterfaceObject { bounds })
                }
            }
        }
    }

    /// Converts a type constructor from HIR into [`TypeConstructor`].
    fn resolve_type_constructor(
        &self,
        ty: &stellar_hir::TypeConstructor,
        generic_parameter_scope: &GenericParameterScope,
        module_scope: &ModuleScope,
    ) -> Option<TypeConstructor> {
        let mut identifiers_iter = ty.path.identifiers.iter();
        let possible_generic_parameter_name = identifiers_iter.next().unwrap();

        if identifiers_iter.next().is_none()
            && ty.arguments.is_empty()
            && generic_parameter_scope.contains(possible_generic_parameter_name.id)
        {
            return Some(TypeConstructor {
                path: Path {
                    identifiers: vec![possible_generic_parameter_name.id],
                },
                arguments: vec![],
            });
        }

        let Some(name_binding) = self.name_resolver.resolve_path_in_module_scope(
            module_scope,
            ty.path.clone(),
            self.identifier_interner,
            self.diagnostics,
        ) else {
            return None;
        };

        let name_binding_kind = name_binding.kind();

        if !name_binding_kind.is_module_item() {
            self.diagnostics.write().add_single_file_diagnostic(
                ty.location.file_path_id,
                ExpectedType::new(ty.location, name_binding_kind),
            );

            return None;
        }

        todo!()
    }

    /// Resolves type arguments.
    fn resolve_type_arguments(
        &self,
        hir: &[stellar_hir::Type],
        signature: bool,
        generic_parameter_scope: &GenericParameterScope,
        module_scope: &ModuleScope,
    ) -> Option<Vec<Type>> {
        hir.iter()
            .map(|ty| self.resolve_type(ty, signature, generic_parameter_scope, module_scope))
            .collect::<Option<_>>()
    }

    pub fn unwrap_type_aliases(&self, constructor: TypeConstructor) -> Type {
        let ty = Type::Constructor(constructor);

        // loop {
        // match ty {
        //     Type::Constructor(constructor) => {
        //         self.signature_analysis_context
        //             .write()
        //             .add_type_alias_to_stack(&constructor.path);

        //         let signature = self.resolve_type_signature_by_path(&constructor.path);
        //     }
        //     _ => ty,
        // }
        // }

        self.signature_analysis_context
            .write()
            .drop_type_alias_stack();

        todo!()
    }

    fn build_type_alias_cycle_diagnostic() -> TypeAliasCycleFound {
        todo!()
    }

    #[allow(clippy::single_match)]
    fn implements(&self, ty: Type, interface: &TypeConstructor) -> bool {
        match ty {
            Type::Constructor(constructor) => {
                let signature = self.resolve_type_signature_by_path(&constructor.path);

                // match signature.as_ref() {
                //     ModuleItemSignature::TypeAlias(alias) => {}
                //     _ => {}
                // }

                todo!()
            }
            _ => false, // implement builtin interfaces later
        }
    }

    pub(crate) fn resolve_interface(
        &self,
        interface: &stellar_hir::TypeConstructor,
        signature: bool,
        generic_parameter_scope: &GenericParameterScope,
        module_scope: &ModuleScope,
    ) -> Option<TypeConstructor> {
        let Some(name_binding) = self.name_resolver.resolve_path_in_module_scope(
            module_scope,
            interface.path.clone(),
            self.identifier_interner,
            self.diagnostics,
        ) else {
            return None;
        };

        let signature_ = self.resolve_signature(name_binding, module_scope)?;

        match signature_.as_ref() {
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
                    signature,
                    generic_parameter_scope,
                    module_scope,
                )?,
            }),
            _ => unreachable!(),
        }
    }

    pub(crate) fn resolve_bounds(
        &self,
        generic_parameter_scope: &GenericParameterScope,
        signature: bool,
        bounds: &[stellar_hir::TypeConstructor],
        module_scope: &ModuleScope,
    ) -> Vec<TypeConstructor> {
        bounds
            .iter()
            .filter_map(|bound| {
                self.resolve_interface(bound, signature, generic_parameter_scope, module_scope)
            })
            .collect()
    }

    fn resolve_type_signature_by_definition_id(
        &self,
        definition_id: DefinitionID,
    ) -> Arc<ModuleItemSignature> {
        todo!()
    }

    fn resolve_type_signature_by_path(
        &self,
        path: &Path,
    ) -> (DefinitionID, Arc<ModuleItemSignature>) {
        todo!()
    }

    fn resolve_interface_signature_by_definition_id(
        &self,
        definition_id: DefinitionID,
    ) -> Arc<ModuleItemSignature> {
        todo!()
    }

    fn resolve_interface_signature_by_path(
        &self,
        path: Path,
    ) -> (DefinitionID, Arc<ModuleItemSignature>) {
        todo!()
    }
}
