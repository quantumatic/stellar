use std::sync::Arc;

use ry_ast::DefinitionID;
use ry_diagnostics::GlobalDiagnostics;
use ry_fx_hash::FxHashMap;
use ry_hir::{Module, ModuleItem};
use ry_interner::{IdentifierInterner, PathID, PathInterner, Symbol};
use ry_name_resolution::NameResolutionContext;
use ry_thir::ty::Type;
use trait_resolution::TraitResolutionContext;

pub mod diagnostics;
pub mod generics_scope;
pub mod trait_resolution;

#[derive(Debug)]
pub struct TypeCheckingContext<'i, 'p, 'd> {
    identifier_interner: &'i mut IdentifierInterner,
    path_interner: &'p PathInterner,
    name_resolution_context: NameResolutionContext,
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
            name_resolution_context: NameResolutionContext::new(),
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
}
