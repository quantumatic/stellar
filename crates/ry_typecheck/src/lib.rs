#![allow(warnings)]

use std::sync::Arc;

use ry_ast::{DefinitionID, TypeConstructor};
use ry_diagnostics::GlobalDiagnostics;
use ry_fx_hash::FxHashMap;
use ry_hir::{Module, ModuleItem};
use ry_interner::{IdentifierInterner, PathID, PathInterner, Symbol};
use ry_name_resolution::{GlobalResolutionContext, ModuleResolutionContext, NameBindingData};
use ry_thir::ty::{self, Type};

pub mod diagnostics;
pub mod generic_parameter_scope;

#[derive(Debug)]
pub struct TypeCheckingContext<'i, 'p, 'd> {
    identifier_interner: &'i mut IdentifierInterner,
    path_interner: &'p PathInterner,
    name_resolution_context: GlobalResolutionContext,
    items: FxHashMap<DefinitionID, ModuleItem>,
    substitutions: FxHashMap<Symbol, Arc<Type>>,
    diagnostics: &'d mut GlobalDiagnostics,
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
            name_resolution_context: GlobalResolutionContext::new(),
            substitutions: FxHashMap::default(),
            items: FxHashMap::default(),
            diagnostics,
        }
    }

    pub fn add_module(&mut self, file_path_id: PathID, hir: Module) {
        for (idx, item) in hir.items.into_iter().enumerate() {
            self.items.insert(
                DefinitionID {
                    index: idx,
                    file_path_id,
                },
                item,
            );
        }
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
            _ => todo!()
        }
    }

    fn lower_type_constructor(&self, ty: ry_ast::TypeConstructor) -> Type {
        todo!()
    }

    fn lower_interface(&self, interface: ry_ast::TypeConstructor) {}
}
