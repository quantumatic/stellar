#![allow(warnings)]

use std::sync::Arc;

use ry_ast::{DefinitionID, TypeConstructor};
use ry_diagnostics::GlobalDiagnostics;
use ry_fx_hash::FxHashMap;
use ry_hir::Module;
use ry_interner::{IdentifierInterner, PathID, PathInterner, Symbol};
use ry_name_resolution::{ModuleScope, Path, ResolutionEnvironment};
use ry_thir::{
    ty::{self, Type},
    InterfaceSignature, ModuleItemSignature,
};

pub mod diagnostics;
pub mod generic_parameter_scope;

#[derive(Debug)]
pub struct TypeCheckingContext<'i, 'p, 'd> {
    identifier_interner: &'i mut IdentifierInterner,
    path_interner: &'p PathInterner,
    resolution_environment: ResolutionEnvironment,
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

    pub fn add_module(&mut self, file_path_id: PathID, path: Path, hir: Module) {
        for (idx, item) in hir.items.into_iter().enumerate() {
            self.items.insert(
                DefinitionID {
                    index: idx,
                    module_path_id: file_path_id,
                },
                ModuleItem::HIR(item),
            );
        }

        //self.resolution_environment
        //    .modules
        //    .insert(file_path_id, ModuleScope);
        self.resolution_environment
            .module_paths
            .insert(file_path_id, path);
    }

    pub fn lower_type(&self, ty: ry_hir::Type) -> Type {
        match ty {
            ry_hir::Type::Constructor(constructor) => self.lower_type_constructor(constructor),
            ry_hir::Type::Tuple { element_types, .. } => Type::Tuple {
                element_types: element_types
                    .into_iter()
                    .map(|element| self.lower_type(element))
                    .collect(),
            },
            ry_hir::Type::Function {
                parameter_types,
                return_type,
                ..
            } => Type::Function {
                parameter_types: parameter_types
                    .into_iter()
                    .map(|parameter| self.lower_type(parameter))
                    .collect(),
                return_type: Box::new(self.lower_type(*return_type)),
            },
            _ => todo!(),
        }
    }

    /// A symbol data, in which types in a definition are processed, once the the
    /// definition is used somewhere else. This approach allows to resolve forward
    /// references.
    fn lower_type_constructor(&self, ty: ry_ast::TypeConstructor) -> Type {
        todo!()
    }

    fn lower_interface(&self, interface: ry_ast::TypeConstructor) {}
}
