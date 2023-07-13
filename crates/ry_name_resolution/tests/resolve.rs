use std::path::PathBuf;

use ry_ast::{IdentifierAst, ImportPath};
use ry_filesystem::span::DUMMY_SPAN;
use ry_fx_hash::FxHashMap;
use ry_interner::Interner;
use ry_name_resolution::{ModuleData, NameBindingData, NameResolutionTree, Path, ProjectData};

#[test]
fn resolve_module() {
    let mut interner = Interner::default();
    let a = interner.get_or_intern("a");
    let b = interner.get_or_intern("b");

    let mut tree = NameResolutionTree::new();

    let child_module_data = ModuleData {
        path: PathBuf::new(),
        docstring: None,
        bindings: FxHashMap::default(),
        submodules: FxHashMap::default(),
        implementations: vec![],
        imports: vec![],
    };

    let mut project_root_module = ModuleData {
        path: PathBuf::new(),
        docstring: None,
        bindings: FxHashMap::default(),
        submodules: FxHashMap::default(),
        implementations: vec![],
        imports: vec![],
    };
    project_root_module.submodules.insert(a, child_module_data);

    let project = ProjectData {
        path: PathBuf::new(),
        root: project_root_module,
        dependencies: vec![],
    };

    tree.projects.insert(a, project);

    assert!(matches!(
        tree.resolve_absolute_path(Path {
            symbols: vec![a, a]
        }),
        Some(NameBindingData::Module(..))
    ));

    assert_eq!(tree.resolve_absolute_path(Path { symbols: vec![a] }), None);
    assert_eq!(tree.resolve_absolute_path(Path { symbols: vec![b] }), None);
}

#[test]
fn import() {
    let mut interner = Interner::default();
    let a = interner.get_or_intern("a");
    let b = interner.get_or_intern("b");
    let c = interner.get_or_intern("c");

    let mut tree = NameResolutionTree::new();

    let child_module_data = ModuleData {
        path: PathBuf::new(),
        docstring: None,
        bindings: FxHashMap::default(),
        submodules: FxHashMap::default(),
        implementations: vec![],
        imports: vec![],
    };

    let import_path1 = ImportPath {
        path: ry_ast::Path {
            span: DUMMY_SPAN,
            identifiers: vec![
                IdentifierAst {
                    span: DUMMY_SPAN,
                    symbol: a,
                },
                IdentifierAst {
                    span: DUMMY_SPAN,
                    symbol: b,
                },
            ],
        },
        r#as: None,
    };

    let import_path2 = ImportPath {
        path: ry_ast::Path {
            span: DUMMY_SPAN,
            identifiers: vec![
                IdentifierAst {
                    span: DUMMY_SPAN,
                    symbol: a,
                },
                IdentifierAst {
                    span: DUMMY_SPAN,
                    symbol: b,
                },
            ],
        },
        r#as: Some(IdentifierAst {
            span: DUMMY_SPAN,
            symbol: c,
        }),
    };

    let mut submodules = FxHashMap::default();
    submodules.insert(b, child_module_data);

    let project_root_module = ModuleData {
        path: PathBuf::new(),
        docstring: None,
        bindings: FxHashMap::default(),
        submodules,
        implementations: vec![],
        imports: vec![(DUMMY_SPAN, &import_path1), (DUMMY_SPAN, &import_path2)],
    };
    let project = ProjectData {
        path: PathBuf::new(),
        root: project_root_module,
        dependencies: vec![],
    };

    tree.projects.insert(a, project);

    assert!(matches!(
        tree.projects
            .get(&a)
            .unwrap()
            .root
            .resolve_path(Path { symbols: vec![b] }, &tree),
        Some(..)
    ));
    assert!(matches!(
        tree.projects
            .get(&a)
            .unwrap()
            .root
            .resolve_path(Path { symbols: vec![c] }, &tree),
        Some(..)
    ));
    assert_eq!(
        tree.projects.get(&a).unwrap().root.resolve_path(
            Path {
                symbols: vec![b, b]
            },
            &tree
        ),
        None
    );
}
