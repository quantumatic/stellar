use std::path::PathBuf;

use ry_ast::{Module, ModuleItem};
use ry_diagnostics::{BuildDiagnostic, Diagnostic};
use ry_fx_hash::FxHashMap;
use ry_interner::Interner;

use crate::{
    diagnostics::{DefinitionInfo, NameResolutionDiagnostic},
    ModuleData, ModuleItemNameBindingData,
};

/// Builds a name resolution tree module node from the module AST
/// and adds diagnostic into the diagnostics if there are two definitions with
/// the same name.
pub fn build_module_node_from_ast<'ast>(
    file_path: impl Into<PathBuf>,
    module: &'ast Module,
    interner: &Interner,
    file_diagnostics: &mut Vec<Diagnostic>,
) -> ModuleData<'ast> {
    let mut bindings = FxHashMap::default();
    let mut implementations = vec![];
    let mut imports = vec![];

    for item in &module.items {
        match item {
            ModuleItem::Impl(r#impl) => {
                implementations.push(r#impl);
            }
            ModuleItem::Import { span, path } => {
                imports.push((*span, path));
            }
            _ => {
                let name = item.name_or_panic();

                if let Some(ModuleItemNameBindingData::NotAnalyzed(previous_item)) =
                    bindings.get(&name)
                {
                    file_diagnostics.push(
                        NameResolutionDiagnostic::ItemDefinedMultipleTimes {
                            name: interner.resolve(name).unwrap().to_owned(),
                            first_definition: DefinitionInfo {
                                span: previous_item.span(),
                                kind: previous_item.kind(),
                            },
                            second_definition: DefinitionInfo {
                                span: item.span(),
                                kind: item.kind(),
                            },
                        }
                        .build(),
                    );
                }

                bindings.insert(name, ModuleItemNameBindingData::NotAnalyzed(item));
            }
        }
    }

    ModuleData {
        path: file_path.into(),
        docstring: module.docstring.as_deref(),
        bindings,
        submodules: FxHashMap::default(),
        implementations,
        imports,
    }
}
