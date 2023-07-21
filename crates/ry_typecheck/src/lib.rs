use std::sync::Arc;

use ry_ast::DefinitionID;
use ry_ast_lowering::TypeVariableGenerator;
use ry_diagnostics::GlobalDiagnostics;
use ry_fx_hash::FxHashMap;
use ry_hir::{ty::Type, Module, ModuleItem};
use ry_interner::{IdentifierInterner, PathID, PathInterner, Symbol};
use ry_name_resolution::NameResolutionContext;
use trait_resolution::TraitResolutionContext;

mod diagnostics;
mod trait_resolution;

#[derive(Debug)]
pub struct TypeCheckingContext<'i, 'p, 'hir, 't, 'd> {
    identifier_interner: &'i mut IdentifierInterner,
    path_interner: &'p PathInterner,
    name_resolution_context: NameResolutionContext,
    trait_resolution_context: TraitResolutionContext,

    items: FxHashMap<DefinitionID, &'hir mut ModuleItem>,

    substitutions: FxHashMap<Symbol, Arc<Type>>,
    type_variable_generator: &'t mut TypeVariableGenerator,

    diagnostics: &'d mut GlobalDiagnostics,
}

impl<'i, 'p, 'hir, 't, 'd> TypeCheckingContext<'i, 'p, 'hir, 't, 'd> {
    pub fn new(
        identifier_interner: &'i mut IdentifierInterner,
        path_interner: &'p PathInterner,
        type_variable_generator: &'t mut TypeVariableGenerator,
        diagnostics: &'d mut GlobalDiagnostics,
    ) -> Self {
        Self {
            identifier_interner,
            path_interner,
            name_resolution_context: NameResolutionContext::new(),
            trait_resolution_context: TraitResolutionContext::new(),
            substitutions: FxHashMap::default(),
            items: FxHashMap::default(),
            type_variable_generator,
            diagnostics,
        }
    }

    pub fn add_module(&mut self, file_path_id: PathID, hir: &'hir mut Module) {
        for (idx, item) in hir.items.iter_mut().enumerate() {
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
