use stellar_ast_lowering::LowerToHir;
use stellar_database::State;
use stellar_interner::{IdentifierID, PathID};
use stellar_parser::parse_module;
use stellar_typechecker::collect_definitions::CollectDefinitions;

#[test]
fn test_enum() {
    let state = State::new();
    let filepath_id = PathID::from("test.sr");
    let source_code = "enum A {}\nenum B {}";

    let hir = &LowerToHir::run_all(
        &state,
        vec![parse_module(filepath_id, source_code, state.diagnostics())],
    );

    CollectDefinitions::run_all(&state, hir);

    assert!(state
        .db_lock()
        .get_module_or_panic(hir[0].0)
        .get_symbol_or_panic(IdentifierID::from("A"))
        .is_enum());
    assert!(state
        .db_lock()
        .get_module_or_panic(hir[0].0)
        .get_symbol_or_panic(IdentifierID::from("B"))
        .is_enum());
    assert!(state.diagnostics_inner().is_ok());
}

#[test]
fn test_duplicate_definition() {
    let state = State::new();
    let filepath_id = PathID::from("test.sr");
    let source_code = "enum A {}\nenum A {}";

    CollectDefinitions::run_all(
        &state,
        &LowerToHir::run_all(
            &state,
            vec![parse_module(filepath_id, source_code, state.diagnostics())],
        ),
    );

    assert_eq!(
        state.diagnostics_inner().file_diagnostics[0].code,
        Some("E005".to_owned())
    );
}

#[test]
fn test_enum_items() {
    let state = State::new();
    let filepath_id = PathID::from("test.sr");
    let source_code = "enum A { A, B, C }";

    let hir = &LowerToHir::run_all(
        &state,
        vec![parse_module(filepath_id, source_code, state.diagnostics())],
    );

    CollectDefinitions::run_all(&state, hir);

    let db = state.db_lock();
    let items = &db
        .get_module_or_panic(hir[0].0)
        .get_symbol_or_panic(IdentifierID::from("A"))
        .get_enum_or_panic(&db)
        .items;

    assert!(items.contains_key(&IdentifierID::from("A")));
    assert!(items.contains_key(&IdentifierID::from("B")));
    assert!(items.contains_key(&IdentifierID::from("C")));
}

#[test]
fn test_function() {
    let state = State::new();
    let filepath_id = PathID::from("test.sr");
    let source_code = "fun a() {}";

    let hir = &LowerToHir::run_all(
        &state,
        vec![parse_module(filepath_id, source_code, state.diagnostics())],
    );

    CollectDefinitions::run_all(&state, hir);

    assert!(state
        .db_lock()
        .get_module_or_panic(hir[0].0)
        .get_symbol_or_panic(IdentifierID::from("a"))
        .is_function());
    assert!(state.diagnostics_inner().is_ok());
}

#[test]
fn test_struct() {
    let state = State::new();
    let filepath_id = PathID::from("test.sr");
    let source_code = "struct A {}";

    let hir = &LowerToHir::run_all(
        &state,
        vec![parse_module(filepath_id, source_code, state.diagnostics())],
    );

    CollectDefinitions::run_all(&state, hir);

    assert!(state
        .db_lock()
        .get_module_or_panic(hir[0].0)
        .get_symbol_or_panic(IdentifierID::from("A"))
        .is_struct());
    assert!(state.diagnostics_inner().is_ok());
}

#[test]
fn test_interface() {
    let state = State::new();
    let filepath_id = PathID::from("test.sr");
    let source_code = "interface A {}";

    let hir = &LowerToHir::run_all(
        &state,
        vec![parse_module(filepath_id, source_code, state.diagnostics())],
    );

    CollectDefinitions::run_all(&state, hir);

    assert!(state
        .db_lock()
        .get_module_or_panic(hir[0].0)
        .get_symbol_or_panic(IdentifierID::from("A"))
        .is_interface());
    assert!(state.diagnostics_inner().is_ok());
}
