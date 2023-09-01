use std::sync::Arc;

use stellar_ast_lowering::LowerToHir;
use stellar_database::State;
use stellar_interner::{IdentifierID, PathID};
use stellar_parser::parse_module;
use stellar_typechecker::{
    collect_definitions::CollectDefinitions, resolve_imports::ResolveImports,
};

#[test]
fn resolve_submodule_import_ok() {
    let state = State::new();
    let submodule_ast = parse_module(&state, IdentifierID::from("b"), PathID::from("a/b.sr"), "");

    let root_ast = parse_module(
        &state,
        IdentifierID::from("a"),
        PathID::from("a/package.sr"),
        "import a.b;",
    );
    root_ast
        .module()
        .add_submodule(&mut state.db_write_lock(), submodule_ast.module());

    state.db_write_lock().add_package(root_ast.module());

    let hir = LowerToHir::run_all(&state, vec![Arc::new(root_ast), Arc::new(submodule_ast)]);

    ResolveImports::run_all(&state, &hir);

    assert!(state.diagnostics_read_lock().is_ok());
}

#[test]
fn resolve_submodule_import_err() {
    let state = State::new();
    let submodule_ast = parse_module(&state, IdentifierID::from("b"), PathID::from("a/b.sr"), "");

    let root_ast = parse_module(
        &state,
        IdentifierID::from("a"),
        PathID::from("a/package.sr"),
        "import a.c;",
    );
    root_ast
        .module()
        .add_submodule(&mut state.db_write_lock(), submodule_ast.module());

    state.db_write_lock().add_package(root_ast.module());

    let hir = LowerToHir::run_all(&state, vec![Arc::new(root_ast), Arc::new(submodule_ast)]);

    ResolveImports::run_all(&state, &hir);

    assert!(state.diagnostics_read_lock().is_fatal());
}

#[test]
fn resolve_module_item_ok() {
    let state = State::new();
    let submodule_ast = parse_module(
        &state,
        IdentifierID::from("b"),
        PathID::from("a/b.sr"),
        "fun foo() {}",
    );

    let root_ast = parse_module(
        &state,
        IdentifierID::from("a"),
        PathID::from("a/package.sr"),
        "import a.b.foo;",
    );
    root_ast
        .module()
        .add_submodule(&mut state.db_write_lock(), submodule_ast.module());

    state.db_write_lock().add_package(root_ast.module());

    let hir = LowerToHir::run_all(&state, vec![Arc::new(root_ast), Arc::new(submodule_ast)]);

    CollectDefinitions::run_all(&state, &hir);
    ResolveImports::run_all(&state, &hir);

    assert!(state.diagnostics_read_lock().is_ok());
}

#[test]
fn resolve_module_item_err1() {
    let state = State::new();
    let submodule_ast = parse_module(
        &state,
        IdentifierID::from("b"),
        PathID::from("a/b.sr"),
        "fun foo() {}",
    );

    let root_ast = parse_module(
        &state,
        IdentifierID::from("a"),
        PathID::from("a/package.sr"),
        "import a.b.foo;
import a.b.foo2;",
    );
    root_ast
        .module()
        .add_submodule(&mut state.db_write_lock(), submodule_ast.module());

    state.db_write_lock().add_package(root_ast.module());

    let hir = LowerToHir::run_all(&state, vec![Arc::new(root_ast), Arc::new(submodule_ast)]);

    CollectDefinitions::run_all(&state, &hir);
    ResolveImports::run_all(&state, &hir);

    assert!(state.diagnostics_read_lock().is_fatal());
}

#[test]
fn resolve_module_item_err2() {
    let state = State::new();
    let submodule_ast = parse_module(
        &state,
        IdentifierID::from("b"),
        PathID::from("a/b.sr"),
        "fun foo() {}",
    );

    let root_ast = parse_module(
        &state,
        IdentifierID::from("a"),
        PathID::from("a/package.sr"),
        "import a.c.foo;",
    );
    root_ast
        .module()
        .add_submodule(&mut state.db_write_lock(), submodule_ast.module());

    state.db_write_lock().add_package(root_ast.module());

    let hir = LowerToHir::run_all(&state, vec![Arc::new(root_ast), Arc::new(submodule_ast)]);

    CollectDefinitions::run_all(&state, &hir);
    ResolveImports::run_all(&state, &hir);

    assert!(state.diagnostics_read_lock().is_fatal());
}
