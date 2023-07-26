use ry_ast::{IdentifierAST, ImportPath};
use ry_filesystem::location::DUMMY_LOCATION;
use ry_fx_hash::FxHashMap;
use ry_interner::{Interner, DUMMY_PATH_ID};
use ry_name_resolution::{
    GlobalResolutionContext, ModuleResolutionContext, NameBindingData, PackageResolutionContext,
};
use ry_thir::ty::Path;

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

    let mut global_context = GlobalResolutionContext::new();

    let child_module_context = ModuleResolutionContext {
        path_id: DUMMY_PATH_ID,
        bindings: FxHashMap::default(),
        submodules: FxHashMap::default(),
        imports: vec![],
    };

    let mut package_root_module_context = ModuleResolutionContext {
        path_id: DUMMY_PATH_ID,
        bindings: FxHashMap::default(),
        submodules: FxHashMap::default(),
        imports: vec![],
    };
    package_root_module_context
        .submodules
        .insert(a, child_module_context);

    let package = PackageResolutionContext {
        path_id: DUMMY_PATH_ID,
        root: package_root_module_context,
        dependencies: vec![],
    };

    global_context.packages.insert(a, package);

    assert!(matches!(
        global_context.resolve_module_item_by_absolute_path(&Path {
            symbols: vec![a, a]
        }),
        Some(NameBindingData::Module(..))
    ));

    assert_eq!(
        global_context.resolve_module_item_by_absolute_path(&Path { symbols: vec![a] }),
        None
    );
    assert_eq!(
        global_context.resolve_module_item_by_absolute_path(&Path { symbols: vec![b] }),
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

    let mut global_context = GlobalResolutionContext::new();

    let child_module_data = ModuleResolutionContext {
        path_id: DUMMY_PATH_ID,
        bindings: FxHashMap::default(),
        submodules: FxHashMap::default(),
        imports: vec![],
    };

    let mut submodules = FxHashMap::default();
    submodules.insert(b, child_module_data);

    let package_root_module = ModuleResolutionContext {
        path_id: DUMMY_PATH_ID,
        bindings: FxHashMap::default(),
        submodules,
        imports: vec![
            (
                DUMMY_LOCATION,
                ImportPath {
                    path: ry_ast::Path {
                        location: DUMMY_LOCATION,
                        identifiers: vec![
                            IdentifierAST {
                                location: DUMMY_LOCATION,
                                symbol: a,
                            },
                            IdentifierAST {
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
                            IdentifierAST {
                                location: DUMMY_LOCATION,
                                symbol: a,
                            },
                            IdentifierAST {
                                location: DUMMY_LOCATION,
                                symbol: b,
                            },
                        ],
                    },
                    r#as: Some(IdentifierAST {
                        location: DUMMY_LOCATION,
                        symbol: c,
                    }),
                },
            ),
        ],
    };
    let package = PackageResolutionContext {
        path_id: DUMMY_PATH_ID,
        root: package_root_module,
        dependencies: vec![],
    };

    global_context.packages.insert(a, package);

    assert!(matches!(
        global_context
            .packages
            .get(&a)
            .unwrap()
            .root
            .resolve_symbol(&global_context, b),
        Some(..)
    ));
    assert!(matches!(
        global_context
            .packages
            .get(&a)
            .unwrap()
            .root
            .resolve_symbol(&global_context, c),
        Some(..)
    ));
}
