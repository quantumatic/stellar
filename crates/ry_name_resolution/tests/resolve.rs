use ry_ast::{IdentifierAst, ImportPath};
use ry_filesystem::{location::DUMMY_LOCATION, path_interner::DUMMY_PATH_ID};
use ry_fx_hash::FxHashMap;
use ry_hir::ty::Path;
use ry_interner::Interner;
use ry_name_resolution::{GlobalContext, ModuleContext, NameBindingData, PackageContext};

/// ```txt
/// a
/// |_ a.ry
///
/// resolve(a.a) = a.a module
/// ```
#[test]
fn resolve_module() {
    let mut interner = Interner::default();
    let a = interner.get_or_intern("a");
    let b = interner.get_or_intern("b");

    let mut tree = GlobalContext::new();

    let child_module_data = ModuleContext {
        path_id: DUMMY_PATH_ID,
        docstring: None,
        bindings: FxHashMap::default(),
        submodules: FxHashMap::default(),
        implementations: vec![],
        imports: vec![],
    };

    let mut package_root_module = ModuleContext {
        path_id: DUMMY_PATH_ID,
        docstring: None,
        bindings: FxHashMap::default(),
        submodules: FxHashMap::default(),
        implementations: vec![],
        imports: vec![],
    };
    package_root_module.submodules.insert(a, child_module_data);

    let package = PackageContext {
        path_id: DUMMY_PATH_ID,
        root: package_root_module,
        dependencies: vec![],
    };

    tree.packages.insert(a, package);

    assert!(matches!(
        tree.resolve_module_item_by_absolute_path(&Path {
            symbols: vec![a, a]
        }),
        Some(NameBindingData::Module(..))
    ));

    assert_eq!(
        tree.resolve_module_item_by_absolute_path(&Path { symbols: vec![a] }),
        None
    );
    assert_eq!(
        tree.resolve_module_item_by_absolute_path(&Path { symbols: vec![b] }),
        None
    );
}

/// ```txt
/// a
/// |_ package.ry
///    |_ `import a.b;`
///    |_ `import a.b as c;`
/// |_ b.ry
///
/// a/package.ry: resolve(b) = a.b, resolve(c) = a.b
/// ```
#[test]
fn import() {
    let mut interner = Interner::default();
    let a = interner.get_or_intern("a");
    let b = interner.get_or_intern("b");
    let c = interner.get_or_intern("c");

    let mut tree = GlobalContext::new();

    let child_module_data = ModuleContext {
        path_id: DUMMY_PATH_ID,
        docstring: None,
        bindings: FxHashMap::default(),
        submodules: FxHashMap::default(),
        implementations: vec![],
        imports: vec![],
    };

    let mut submodules = FxHashMap::default();
    submodules.insert(b, child_module_data);

    let package_root_module = ModuleContext {
        path_id: DUMMY_PATH_ID,
        docstring: None,
        bindings: FxHashMap::default(),
        submodules,
        implementations: vec![],
        imports: vec![
            (
                DUMMY_LOCATION,
                ImportPath {
                    path: ry_ast::Path {
                        location: DUMMY_LOCATION,
                        identifiers: vec![
                            IdentifierAst {
                                location: DUMMY_LOCATION,
                                symbol: a,
                            },
                            IdentifierAst {
                                location: DUMMY_LOCATION,
                                symbol: b,
                            },
                        ],
                    },
                    r#as: None,
                },
            ),
            (
                DUMMY_LOCATION,
                ImportPath {
                    path: ry_ast::Path {
                        location: DUMMY_LOCATION,
                        identifiers: vec![
                            IdentifierAst {
                                location: DUMMY_LOCATION,
                                symbol: a,
                            },
                            IdentifierAst {
                                location: DUMMY_LOCATION,
                                symbol: b,
                            },
                        ],
                    },
                    r#as: Some(IdentifierAst {
                        location: DUMMY_LOCATION,
                        symbol: c,
                    }),
                },
            ),
        ],
    };
    let package = PackageContext {
        path_id: DUMMY_PATH_ID,
        root: package_root_module,
        dependencies: vec![],
    };

    tree.packages.insert(a, package);

    assert!(matches!(
        tree.packages
            .get(&a)
            .unwrap()
            .root
            .resolve_module_item_path(&Path { symbols: vec![b] }, &tree),
        Some(..)
    ));
    assert!(matches!(
        tree.packages
            .get(&a)
            .unwrap()
            .root
            .resolve_module_item_path(&Path { symbols: vec![c] }, &tree),
        Some(..)
    ));
    assert_eq!(
        tree.packages
            .get(&a)
            .unwrap()
            .root
            .resolve_module_item_path(
                &Path {
                    symbols: vec![b, b]
                },
                &tree
            ),
        None
    );
}
