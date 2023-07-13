use std::path::PathBuf;

use ry_ast::{Function, FunctionSignature, Module, ModuleItem, TypeAlias};
use ry_diagnostics::{BuildDiagnostic, Diagnostic};
use ry_fx_hash::FxHashMap;
use ry_interner::{Interner, Symbol};

use crate::{
    diagnostics::{DefinitionInfo, NameResolutionDiagnostics},
    ModuleData, ModuleItemNameBindingData,
};

pub fn module_item_name(item: &ModuleItem) -> Symbol {
    match item {
        ModuleItem::Enum { name, .. }
        | ModuleItem::Function(Function {
            signature: FunctionSignature { name, .. },
            ..
        })
        | ModuleItem::Struct { name, .. }
        | ModuleItem::TupleLikeStruct { name, .. }
        | ModuleItem::Trait { name, .. }
        | ModuleItem::TypeAlias(TypeAlias { name, .. }) => name.symbol,
        ModuleItem::Import { .. } | ModuleItem::Impl(..) => unreachable!(),
    }
}

pub fn build_resolution_tree_node_from_ast<'ast>(
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
                let name = module_item_name(item);

                match bindings.get(&name) {
                    Some(ModuleItemNameBindingData::NotAnalyzed(previous_item)) => {
                        file_diagnostics.push(
                            NameResolutionDiagnostics::OverwrittingModuleItem {
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
                    _ => {}
                }

                bindings.insert(
                    module_item_name(item),
                    ModuleItemNameBindingData::NotAnalyzed(item),
                );
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
