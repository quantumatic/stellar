use std::{
    collections::HashMap,
    fs, io,
    path::{self, Path},
};

use ry_ast::{Function, Item, TypeAlias};
use ry_diagnostics::{Diagnostic, GlobalDiagnostics};
use ry_filesystem::path_resolver::ProjectPathResolver;
use ry_interner::{symbols, Interner, Symbol};
use ry_manifest::parse_manifest;
use ry_parser::read_and_parse_module;

use crate::{ModuleItem, ModuleNode, ProjectNode, ResolutionTree};

fn get_symbol_corresponding_to_item(item: &Item) -> Symbol {
    match item {
        Item::Enum { name, .. }
        | Item::Function(Function { name, .. })
        | Item::Trait { name, .. }
        | Item::Struct { name, .. }
        | Item::TupleLikeStruct { name, .. }
        | Item::TypeAlias(TypeAlias { name, .. }) => name.symbol,
        Item::Import { .. } | Item::Impl { .. } => symbols::UNDERSCORE, // place holder
    }
}

pub fn parse_and_build_resolution_tree_module_node_for_file<P>(
    filepath: P,
    diagnostics: &mut Vec<Diagnostic>,
    interner: &mut Interner,
) -> Result<ModuleNode, io::Error>
where
    P: AsRef<path::Path>,
{
    let ast = read_and_parse_module(filepath, diagnostics, interner)?;
    let mut items = HashMap::new();

    for item in ast.items {
        items.insert(
            get_symbol_corresponding_to_item(&item),
            ModuleItem::NotAnalyzedItem(item),
        );
    }

    Ok(ModuleNode {
        docstring: ast.docstring,
        imports: vec![],
        items,
    })
}

pub fn build_resolution_tree<P>(
    project_root: P,
    diagnostics: &mut GlobalDiagnostics,
    interner: &mut Interner,
) where
    P: AsRef<Path>,
{
    let project_root = project_root.as_ref();

    let path_resolver = ProjectPathResolver { root: project_root };

    let Ok(manifest_source) = fs::read_to_string(path_resolver.manifest()) else {
        diagnostics
            .context_free_diagnostics
            .push(Diagnostic::error().with_message(format!(
                "cannot find project's manifest file in {}",
                project_root.display()
            )));
        return;
    };

    match parse_manifest(manifest_source) {
        Ok(manifest) => {
            let project_name = manifest.project.name;

            let mut tree = ResolutionTree::new();
            tree.projects.insert(
                interner.get_or_intern(project_name),
                build_resolution_tree_project_node(path_resolver, diagnostics, interner),
            );
        }
        Err(err) => diagnostics
            .context_free_diagnostics
            .push(Diagnostic::error().with_message(format!(
                "cannot parse projecct's manifest file due to the error: {err}"
            ))),
    }
}

fn build_resolution_tree_project_node(
    project_path_resolver: ProjectPathResolver,
    diagnostics: &mut GlobalDiagnostics,
    interner: &mut Interner,
) -> ProjectNode {
    let mut tree = ProjectNode::new();

    let Ok(source_directory_reader) = fs::read_dir(project_path_resolver.src_directory()) else {
        diagnostics.context_free_diagnostics.push(Diagnostic::error().with_message(
            format!("cannot read source directory in {}",
            project_path_resolver.root.display())));
        return ProjectNode {
            modules: HashMap::new(),
        };
    };

    for module in source_directory_reader.flatten() {
        if module.path().is_file() && module.path().ends_with(".ry") {
            let Some(node) = build_resolution_tree_module_node_for_file(
                module.path(),
                diagnostics,
                interner,
            ) else {
                continue;
            };

            tree.modules.insert(node.0, node.1);
        } else {
            let Some(node) = build_resolution_tree_module_node_for_folder(
                    module.path(),
                    diagnostics,
                    interner,
            ) else {
                continue;
            };

            tree.modules.insert(node.0, node.1);
        }
    }

    tree
}

fn build_resolution_tree_module_node_for_file<P>(
    path: P,
    diagnostics: &mut GlobalDiagnostics,
    interner: &mut Interner,
) -> Option<(Symbol, ModuleNode)>
where
    P: AsRef<path::Path>,
{
    let Ok(path) = path.as_ref().canonicalize() else {
        return None;
    };

    let mut file_diagnostics = vec![];
    let Ok(module_node) = parse_and_build_resolution_tree_module_node_for_file(
        path.clone(),
        &mut file_diagnostics,
        interner,
    ) else {
        return None;
    };

    if !file_diagnostics.is_empty() {
        diagnostics.add_file_diagnostics(path.clone(), file_diagnostics);
    }

    let file_name = path.file_name()?.to_str()?;

    Some((
        interner.get_or_intern(&file_name[..file_name.len() - 3]),
        module_node,
    ))
}

fn build_resolution_tree_module_node_for_folder<P>(
    path: P,
    diagnostics: &mut GlobalDiagnostics,
    interner: &mut Interner,
) -> Option<(Symbol, ModuleNode)>
where
    P: AsRef<path::Path>,
{
    let Ok(path) = path.as_ref().canonicalize() else {
        return None;
    };

    let module_name = path.file_name()?.to_str()?;

    if !module_name.chars().all(|c| c.is_alphanumeric()) {
        return None;
    }

    let module_name_symbol = interner.get_or_intern(module_name);

    let mut file_diagnostics = vec![];

    let mod_file_path = path.join("mod.ry");

    let mut module_node = match parse_and_build_resolution_tree_module_node_for_file(
        mod_file_path.clone(),
        &mut file_diagnostics,
        interner,
    ) {
        Ok(module_node) => module_node,
        Err(..) => ModuleNode {
            docstring: None,
            imports: vec![],
            items: HashMap::new(),
        },
    };

    if !file_diagnostics.is_empty() {
        diagnostics.add_file_diagnostics(mod_file_path, file_diagnostics);
    }

    let Ok(module_directory_reader) = fs::read_dir(path) else {
        return Some((module_name_symbol, module_node));
    };

    for module in module_directory_reader.flatten() {
        if module.path().is_file() && module.path().ends_with(".ry") {
            let Some(node) = build_resolution_tree_module_node_for_file(
                module.path(),
                diagnostics,
                interner,
            ) else {
                continue;
            };

            module_node.items.insert(node.0, ModuleItem::Module(node.1));
        } else {
            let Some(child_module_node) = build_resolution_tree_module_node_for_folder(
                module.path(),
                diagnostics,
                interner,
            ) else {
                continue;
            };

            module_node
                .items
                .insert(child_module_node.0, ModuleItem::Module(child_module_node.1));
        }
    }

    Some((module_name_symbol, module_node))
}
