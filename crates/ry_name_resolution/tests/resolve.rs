use std::iter;

use parking_lot::RwLock;
use ry_ast::{dummy_identifier, dummy_path};
use ry_diagnostics::Diagnostics;
use ry_filesystem::location::DUMMY_LOCATION;
use ry_fx_hash::FxHashMap;
use ry_interner::{IdentifierInterner, PathInterner};
use ry_name_resolution::{path, ModuleID, ModuleScope, NameBinding, NameResolver, PackageID};

/// ```txt
/// a
/// |_ a.ry
/// |_ package.ry
///
/// resolve(a.a) = a.a module
/// ```
#[test]
fn resolve_module() {
    let mut identifier_interner = IdentifierInterner::new();
    let mut path_interner = PathInterner::new();

    let (a, b) = identifier_interner.get_or_intern_tuple(["a", "b"]).unwrap();

    let (package_root_module_path_id, child_module_path_id) = path_interner
        .get_or_intern_tuple(["a/package.ry", "a/a.ry"])
        .unwrap();

    let package_root_module_id = ModuleID(package_root_module_path_id);
    let child_module_id = ModuleID(child_module_path_id);

    let mut name_resolver = NameResolver::new();
    let diagnostics = RwLock::new(Diagnostics::new());

    name_resolver.add_package(PackageID(a), package_root_module_id);
    name_resolver.add_module_scopes([
        (
            package_root_module_id,
            ModuleScope {
                name: a,
                id: package_root_module_id,
                path: path!(a),
                bindings: FxHashMap::from_iter(iter::once((
                    a,
                    NameBinding::Module(child_module_id),
                ))),
                enums: FxHashMap::default(),
                imports: FxHashMap::default(),
            },
        ),
        (
            child_module_id,
            ModuleScope {
                name: a,
                id: child_module_id,
                path: path!(a, a),
                bindings: FxHashMap::default(),
                enums: FxHashMap::default(),
                imports: FxHashMap::default(),
            },
        ),
    ]);

    assert_eq!(
        name_resolver.resolve_path(dummy_path!(a, a), &identifier_interner, &diagnostics),
        Some(NameBinding::Module(child_module_id))
    );
    assert_eq!(
        name_resolver.resolve_path(dummy_path!(b), &identifier_interner, &diagnostics),
        None,
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
    let mut identifier_interner = IdentifierInterner::new();
    let (a, b, c) = identifier_interner
        .get_or_intern_tuple(["a", "b", "c"])
        .unwrap();

    let mut path_interner = PathInterner::new();

    let (package_root_module_path_id, child_module_path_id) = path_interner
        .get_or_intern_tuple(["a/package.ry", "a/b.ry"])
        .unwrap();

    let package_root_module_id = ModuleID(package_root_module_path_id);
    let child_module_id = ModuleID(child_module_path_id);

    let mut name_resolver = NameResolver::new();
    let diagnostics = RwLock::new(Diagnostics::new());

    name_resolver.add_package(PackageID(a), package_root_module_id);
    name_resolver.add_module_scopes([
        (
            package_root_module_id,
            ModuleScope {
                name: a,
                id: package_root_module_id,
                path: path!(a),
                bindings: FxHashMap::from_iter(iter::once((
                    b,
                    NameBinding::Module(child_module_id),
                ))),
                imports: FxHashMap::from_iter(
                    [(b, dummy_path!(a, b)), (c, dummy_path!(a, b))].into_iter(),
                ),
                enums: FxHashMap::default(),
            },
        ),
        (
            child_module_id,
            ModuleScope {
                name: b,
                id: child_module_id,
                path: path!(a, b),
                bindings: FxHashMap::default(),
                imports: FxHashMap::default(),
                enums: FxHashMap::default(),
            },
        ),
    ]);

    name_resolver.resolve_imports(&identifier_interner, &diagnostics);

    assert_eq!(
        name_resolver.resolve_identifier_in_module_scope(
            name_resolver.resolve_module_scope_or_panic(package_root_module_id),
            dummy_identifier!(b),
            &identifier_interner,
            &diagnostics,
        ),
        Some(NameBinding::Module(child_module_id))
    );
    assert_eq!(
        name_resolver.resolve_identifier_in_module_scope(
            name_resolver.resolve_module_scope_or_panic(package_root_module_id),
            dummy_identifier!(c),
            &identifier_interner,
            &diagnostics
        ),
        Some(NameBinding::Module(child_module_id))
    );
}
