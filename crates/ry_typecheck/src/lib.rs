use ry_ast::DefinitionID;
use ry_ast_lowering::TypeVariableGenerator;
use ry_diagnostics::GlobalDiagnostics;
use ry_filesystem::path_interner::{PathID, PathInterner};
use ry_fx_hash::FxHashMap;
use ry_hir::{ty::Type, Module, ModuleItem};
use ry_interner::{Interner, Symbol};
use ry_name_resolution::NameResolutionContext;
use trait_resolution::TraitResolutionContext;

mod trait_resolution;

#[derive(Debug)]
pub struct TypeCheckingContext<'hir, 'identifier_interner, 'path_interner, 'diagnostics> {
    identifier_interner: &'identifier_interner mut Interner,
    path_interner: &'path_interner PathInterner,
    name_resolution_context: NameResolutionContext,
    trait_resolution_context: TraitResolutionContext,

    items: FxHashMap<DefinitionID, &'hir mut ModuleItem>,
    module_docstrings: FxHashMap<PathID, Option<&'hir str>>,

    substitutions: FxHashMap<Symbol, Type>,
    type_variable_generator: TypeVariableGenerator,

    diagnostics: &'diagnostics mut GlobalDiagnostics,
}

impl<'hir, 'identifier_interner, 'path_interner, 'diagnostics>
    TypeCheckingContext<'hir, 'identifier_interner, 'path_interner, 'diagnostics>
{
    pub fn new(
        identifier_interner: &'identifier_interner mut Interner,
        path_interner: &'path_interner PathInterner,
        diagnostics: &'diagnostics mut GlobalDiagnostics,
    ) -> Self {
        Self {
            identifier_interner,
            path_interner,
            name_resolution_context: NameResolutionContext::new(),
            trait_resolution_context: TraitResolutionContext::new(),
            substitutions: FxHashMap::default(),
            items: FxHashMap::default(),
            module_docstrings: FxHashMap::default(),
            type_variable_generator: TypeVariableGenerator::new(),
            diagnostics,
        }
    }

    pub fn add_module(&mut self, file_path_id: PathID, hir: &'hir mut Module) {
        self.module_docstrings
            .insert(file_path_id, hir.docstring.as_deref());

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
