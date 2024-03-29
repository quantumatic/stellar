use stellar_ast_lowering::LowerToHir;
use stellar_database::{PackageData, State};
use stellar_interner::{IdentifierId, PathId, DUMMY_PATH_ID};
use stellar_parser::parse_module;
use stellar_typechecker::{
    resolution::collect_definitions::CollectDefinitions,
    resolution::resolve_imports::ResolveImports,
};

#[test]
fn resolve_submodule_import_ok() {
    let mut state = State::new();

    let package = PackageData::alloc(state.db_mut(), IdentifierId::from("a"), DUMMY_PATH_ID);
    let submodule = parse_module(
        &mut state,
        package,
        IdentifierId::from("b"),
        PathId::from("a/b.sr"),
        "",
    );
    let root = parse_module(
        &mut state,
        package,
        IdentifierId::from("a"),
        PathId::from("a/package.sr"),
        "import a.b;",
    );

    package.set_root_module(state.db_mut(), root.module());

    root.module()
        .add_submodule(state.db_mut(), submodule.module());

    let hir = LowerToHir::run_all(&mut state, vec![root, submodule]);

    ResolveImports::run_all(&mut state, &hir);

    assert!(state.diagnostics().is_ok());
}

#[test]
fn resolve_submodule_import_err() {
    let mut state = State::new();

    let package = PackageData::alloc(state.db_mut(), IdentifierId::from("a"), DUMMY_PATH_ID);
    let submodule = parse_module(
        &mut state,
        package,
        IdentifierId::from("b"),
        PathId::from("a/b.sr"),
        "",
    );
    let root = parse_module(
        &mut state,
        package,
        IdentifierId::from("a"),
        PathId::from("a/package.sr"),
        "import a.c;",
    );

    package.set_root_module(state.db_mut(), root.module());
    root.module()
        .add_submodule(state.db_mut(), submodule.module());

    let hir = LowerToHir::run_all(&mut state, vec![root, submodule]);

    ResolveImports::run_all(&mut state, &hir);

    assert!(state.diagnostics().is_fatal());
}

#[test]
fn resolve_module_item_ok() {
    let mut state = State::new();

    let package = PackageData::alloc(state.db_mut(), IdentifierId::from("a"), DUMMY_PATH_ID);
    let submodule = parse_module(
        &mut state,
        package,
        IdentifierId::from("b"),
        PathId::from("a/b.sr"),
        "fun foo() {}",
    );
    let root = parse_module(
        &mut state,
        package,
        IdentifierId::from("a"),
        PathId::from("a/package.sr"),
        "import a.b.foo;",
    );

    package.set_root_module(state.db_mut(), root.module());
    root.module()
        .add_submodule(state.db_mut(), submodule.module());

    let hir = LowerToHir::run_all(&mut state, vec![root, submodule]);

    CollectDefinitions::run_all(&mut state, &hir);
    ResolveImports::run_all(&mut state, &hir);

    assert!(state.diagnostics().is_ok());
}

#[test]
fn resolve_module_item_err1() {
    let mut state = State::new();

    let package = PackageData::alloc(state.db_mut(), IdentifierId::from("a"), DUMMY_PATH_ID);
    let submodule = parse_module(
        &mut state,
        package,
        IdentifierId::from("b"),
        PathId::from("a/b.sr"),
        "fun foo() {}",
    );
    let root = parse_module(
        &mut state,
        package,
        IdentifierId::from("a"),
        PathId::from("a/package.sr"),
        "import a.b.foo;
import a.b.foo2;",
    );

    package.set_root_module(state.db_mut(), root.module());
    root.module()
        .add_submodule(state.db_mut(), submodule.module());

    let hir = LowerToHir::run_all(&mut state, vec![root, submodule]);

    CollectDefinitions::run_all(&mut state, &hir);
    ResolveImports::run_all(&mut state, &hir);

    assert!(state.diagnostics().is_fatal());
}

#[test]
fn resolve_module_item_err2() {
    let mut state = State::new();

    let package = PackageData::alloc(state.db_mut(), IdentifierId::from("a"), DUMMY_PATH_ID);
    let submodule = parse_module(
        &mut state,
        package,
        IdentifierId::from("b"),
        PathId::from("a/b.sr"),
        "fun foo() {}",
    );
    let root = parse_module(
        &mut state,
        package,
        IdentifierId::from("a"),
        PathId::from("a/package.sr"),
        "import a.c.foo;",
    );

    package.set_root_module(state.db_mut(), root.module());
    root.module()
        .add_submodule(state.db_mut(), submodule.module());

    let hir = LowerToHir::run_all(&mut state, vec![root, submodule]);

    CollectDefinitions::run_all(&mut state, &hir);
    ResolveImports::run_all(&mut state, &hir);

    assert!(state.diagnostics().is_fatal());
}

#[test]
fn resolve_enum_item_ok() {
    let mut state = State::new();

    let package = PackageData::alloc(state.db_mut(), IdentifierId::from("a"), DUMMY_PATH_ID);
    let submodule = parse_module(
        &mut state,
        package,
        IdentifierId::from("b"),
        PathId::from("a/b.sr"),
        "enum Result[T, E] { Ok(T), Err(E) }",
    );
    let root = parse_module(
        &mut state,
        package,
        IdentifierId::from("a"),
        PathId::from("a/package.sr"),
        "import a.b.Result;
import a.b.Result.Ok;
import a.b.Result.Err;",
    );

    package.set_root_module(state.db_mut(), root.module());
    root.module()
        .add_submodule(state.db_mut(), submodule.module());

    let hir = LowerToHir::run_all(&mut state, vec![root, submodule]);

    CollectDefinitions::run_all(&mut state, &hir);
    ResolveImports::run_all(&mut state, &hir);

    assert!(state.diagnostics().is_ok());
}

#[test]
fn resolve_enum_item_err1() {
    let mut state = State::new();

    let package = PackageData::alloc(state.db_mut(), IdentifierId::from("a"), DUMMY_PATH_ID);
    let submodule = parse_module(
        &mut state,
        package,
        IdentifierId::from("b"),
        PathId::from("a/b.sr"),
        "enum Result[T, E] { Ok(T), Err(E) }",
    );
    let root = parse_module(
        &mut state,
        package,
        IdentifierId::from("a"),
        PathId::from("a/package.sr"),
        "import a.b.Result.Foo;",
    );

    package.set_root_module(state.db_mut(), root.module());
    root.module()
        .add_submodule(state.db_mut(), submodule.module());

    let hir = LowerToHir::run_all(&mut state, vec![root, submodule]);

    CollectDefinitions::run_all(&mut state, &hir);
    ResolveImports::run_all(&mut state, &hir);

    assert!(state.diagnostics().is_fatal());
}

#[test]
fn resolve_enum_item_err2() {
    let mut state = State::new();

    let package = PackageData::alloc(state.db_mut(), IdentifierId::from("a"), DUMMY_PATH_ID);
    let submodule = parse_module(
        &mut state,
        package,
        IdentifierId::from("b"),
        PathId::from("a/b.sr"),
        "enum Result[T, E] { Ok(T), Err(E) }",
    );
    let root = parse_module(
        &mut state,
        package,
        IdentifierId::from("a"),
        PathId::from("a/package.sr"),
        "import a.b.Result.Ok.Foo;",
    );

    package.set_root_module(state.db_mut(), root.module());
    root.module()
        .add_submodule(state.db_mut(), submodule.module());

    let hir = LowerToHir::run_all(&mut state, vec![root, submodule]);

    CollectDefinitions::run_all(&mut state, &hir);
    ResolveImports::run_all(&mut state, &hir);

    assert!(state.diagnostics().is_fatal());
}

#[test]
fn resolve_name_in_module_items_except_enums() {
    let mut state = State::new();

    let package = PackageData::alloc(state.db_mut(), IdentifierId::from("a"), DUMMY_PATH_ID);
    let submodule = parse_module(
        &mut state,
        package,
        IdentifierId::from("b"),
        PathId::from("a/b.sr"),
        "fun foo() {}",
    );
    let root = parse_module(
        &mut state,
        package,
        IdentifierId::from("a"),
        PathId::from("a/package.sr"),
        "import a.b.foo.foo;",
    );

    package.set_root_module(state.db_mut(), root.module());
    root.module()
        .add_submodule(state.db_mut(), submodule.module());

    let hir = LowerToHir::run_all(&mut state, vec![root, submodule]);

    CollectDefinitions::run_all(&mut state, &hir);
    ResolveImports::run_all(&mut state, &hir);

    assert!(state.diagnostics().is_fatal());
}

#[test]
fn importing_package() {
    let mut state = State::new();

    let package = PackageData::alloc(state.db_mut(), IdentifierId::from("a"), DUMMY_PATH_ID);
    let submodule = parse_module(
        &mut state,
        package,
        IdentifierId::from("b"),
        PathId::from("a/b.sr"),
        "import a;",
    );
    let root = parse_module(
        &mut state,
        package,
        IdentifierId::from("a"),
        PathId::from("a/package.sr"),
        "",
    );

    package.set_root_module(state.db_mut(), root.module());
    root.module()
        .add_submodule(state.db_mut(), submodule.module());

    let hir = LowerToHir::run_all(&mut state, vec![root, submodule]);

    CollectDefinitions::run_all(&mut state, &hir);
    ResolveImports::run_all(&mut state, &hir);

    assert!(state.diagnostics().is_fatal());
}
