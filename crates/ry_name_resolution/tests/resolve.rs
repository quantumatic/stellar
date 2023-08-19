use parking_lot::RwLock;
use ry_ast::{IdentifierAST, Path};
use ry_diagnostics::Diagnostics;
use ry_filesystem::location::DUMMY_LOCATION;
use ry_fx_hash::FxHashMap;
use ry_interner::{IdentifierInterner, PathInterner};
use ry_name_resolution::{ModuleID, ModuleScope, NameBinding, PackageID, ResolutionEnvironment};

macro_rules! dummy_identifier {
    ($id:expr) => {
        IdentifierAST {
            id: $id,
            location: DUMMY_LOCATION,
        }
    };
}

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
    let a = identifier_interner.get_or_intern("a");
    let b = identifier_interner.get_or_intern("b");

    let mut path_interner = PathInterner::new();

    let package_root_module_path_id = path_interner.get_or_intern("a/package.ry");
    let child_module_path_id = path_interner.get_or_intern("a/a.ry");

    let package_root_module_id = ModuleID(package_root_module_path_id);
    let child_module_id = ModuleID(child_module_path_id);

    let mut environment = ResolutionEnvironment::new();
    let diagnostics = RwLock::new(Diagnostics::new());

    let mut package_root_module_scope = ModuleScope {
        name: a,
        id: package_root_module_id,
        bindings: FxHashMap::default(),
        enums: FxHashMap::default(),
        imports: FxHashMap::default(),
    };

    package_root_module_scope
        .bindings
        .insert(a, NameBinding::Module(child_module_id));

    let child_module_scope = ModuleScope {
        name: a,
        id: child_module_id,
        bindings: FxHashMap::default(),
        enums: FxHashMap::default(),
        imports: FxHashMap::default(),
    };

    environment
        .packages_root_modules
        .insert(PackageID(a), package_root_module_id);
    environment
        .module_scopes
        .insert(package_root_module_id, package_root_module_scope);
    environment
        .module_scopes
        .insert(child_module_id, child_module_scope);

    assert_eq!(
        environment.resolve_path(
            Path {
                location: DUMMY_LOCATION,
                identifiers: vec![dummy_identifier!(a), dummy_identifier!(a)],
            },
            &identifier_interner,
            &diagnostics
        ),
        Some(NameBinding::Module(child_module_id))
    );

    assert_eq!(
        environment.resolve_path(
            Path {
                location: DUMMY_LOCATION,
                identifiers: vec![dummy_identifier!(b)],
            },
            &identifier_interner,
            &diagnostics
        ),
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
    let a = identifier_interner.get_or_intern("a");
    let b = identifier_interner.get_or_intern("b");
    let c = identifier_interner.get_or_intern("c");

    let mut path_interner = PathInterner::new();

    let package_root_module_path_id = path_interner.get_or_intern("a/package.ry");
    let child_module_path_id = path_interner.get_or_intern("a/b.ry");

    let package_root_module_id = ModuleID(package_root_module_path_id);
    let child_module_id = ModuleID(child_module_path_id);

    let mut environment = ResolutionEnvironment::new();
    let diagnostics = RwLock::new(Diagnostics::new());

    let mut package_root_module_scope = ModuleScope {
        name: a,
        id: package_root_module_id,
        bindings: FxHashMap::default(),
        imports: FxHashMap::default(),
        enums: FxHashMap::default(),
    };
    package_root_module_scope
        .bindings
        .insert(b, NameBinding::Module(child_module_id));

    let child_module_scope = ModuleScope {
        name: b,
        id: child_module_id,
        bindings: FxHashMap::default(),
        imports: FxHashMap::default(),
        enums: FxHashMap::default(),
    };

    package_root_module_scope.imports.insert(
        b,
        Path {
            location: DUMMY_LOCATION,
            identifiers: vec![dummy_identifier!(a), dummy_identifier!(b)],
        },
    );
    package_root_module_scope.imports.insert(
        c,
        Path {
            location: DUMMY_LOCATION,
            identifiers: vec![dummy_identifier!(a), dummy_identifier!(b)],
        },
    );

    environment
        .packages_root_modules
        .insert(PackageID(a), package_root_module_id);
    environment
        .module_scopes
        .insert(package_root_module_id, package_root_module_scope);
    environment
        .module_scopes
        .insert(child_module_id, child_module_scope);

    environment.resolve_imports(&identifier_interner, &diagnostics);

    assert_eq!(
        environment
            .module_scopes
            .get(&package_root_module_id)
            .unwrap()
            .resolve(
                dummy_identifier!(b),
                &identifier_interner,
                &diagnostics,
                &environment
            ),
        Some(NameBinding::Module(child_module_id))
    );
    assert_eq!(
        environment
            .module_scopes
            .get(&package_root_module_id)
            .unwrap()
            .resolve(
                dummy_identifier!(c),
                &identifier_interner,
                &diagnostics,
                &environment
            ),
        Some(NameBinding::Module(child_module_id))
    );
}
