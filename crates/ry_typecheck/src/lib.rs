#![allow(warnings)]

use std::sync::Arc;

use ry_ast::DefinitionID;
use ry_diagnostics::GlobalDiagnostics;
use ry_fx_hash::FxHashMap;
use ry_hir::{Module, ModuleItem};
use ry_interner::{IdentifierInterner, PathID, PathInterner, Symbol};
use ry_name_resolution::{GlobalResolutionContext, ModuleResolutionContext, NameBindingData};
use ry_thir::ty::{self, Type};
use trait_resolution::TraitResolutionContext;

pub mod diagnostics;
pub mod generic_parameter_scope;
pub mod trait_resolution;

#[derive(Debug)]
pub struct TypeCheckingContext<'i, 'p, 'd> {
    identifier_interner: &'i mut IdentifierInterner,
    path_interner: &'p PathInterner,
    name_resolution_context: GlobalResolutionContext,
    trait_resolution_context: TraitResolutionContext,
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
            trait_resolution_context: TraitResolutionContext::new(),
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

    pub fn resolve_trait_name(
        &self,
        module_context: &ModuleResolutionContext,
        path: ry_hir::Path,
    ) -> ty::Path {
        let mut name_binding = NameBindingData::Module(module_context);

        for identifier in path.identifiers {
            name_binding = match name_binding {
                NameBindingData::Package(package) => package
                    .root
                    .resolve_symbol(&self.name_resolution_context, identifier.symbol)
                    .unwrap(),
                NameBindingData::Module(module) => module
                    .resolve_symbol(&self.name_resolution_context, identifier.symbol)
                    .unwrap(),
                item @ NameBindingData::Item(..) => {
                    break;

                    item
                }
            }
        }

        todo!()
    }
}
