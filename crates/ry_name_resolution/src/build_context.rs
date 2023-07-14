//! Builds a resolution context out of a parsed AST.

use std::{io, path::PathBuf};

use ry_ast::{Module, ModuleItem};
use ry_diagnostics::{BuildDiagnostic, Diagnostic};
use ry_fx_hash::FxHashMap;
use ry_interner::Interner;
use ry_parser::read_and_parse_module;

use crate::{
    diagnostics::{DefinitionInfo, NameResolutionDiagnostic},
    ModuleContext, ModuleItemNameBindingData,
};

/// Builds a module context from the module's AST and adds diagnostic into
/// the diagnostics if there are two definitions with the same name.
pub fn build_module_context_from_ast(
    file_path: impl Into<PathBuf>,
    module: Module,
    interner: &Interner,
    file_diagnostics: &mut Vec<Diagnostic>,
) -> ModuleContext {
    let mut bindings = FxHashMap::default();
    let mut implementations = vec![];
    let mut imports = vec![];

    for item in module.items {
        match item {
            ModuleItem::Impl(r#impl) => {
                implementations.push(r#impl);
            }
            ModuleItem::Import { span, path } => {
                imports.push((span, path));
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

    ModuleContext {
        path: file_path.into(),
        docstring: module.docstring,
        bindings,
        submodules: FxHashMap::default(),
        implementations,
        imports,
    }
}

/// Reads, parses and builds a module context.
#[inline]
pub fn read_and_build_module_context(
    file_path: impl Into<PathBuf>,
    interner: &mut Interner,
    file_diagnostics: &mut Vec<Diagnostic>,
) -> Result<ModuleContext, io::Error> {
    let file_path = file_path.into();

    let module = read_and_parse_module(file_path.clone(), file_diagnostics, interner)?;

    Ok(build_module_context_from_ast(
        file_path,
        module,
        interner,
        file_diagnostics,
    ))
}
