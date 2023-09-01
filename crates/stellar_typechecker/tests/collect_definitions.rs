use stellar_ast_lowering::LowerToHir;
use stellar_database::State;
use stellar_interner::{IdentifierID, PathID, DUMMY_IDENTIFIER_ID};
use stellar_parser::parse_module;
use stellar_typechecker::collect_definitions::CollectDefinitions;

#[test]
fn test_enum() {
    let state = State::new();
    let filepath = PathID::from("test.sr");
    let source_code = "enum A {}\nenum B {}";

    let ast = parse_module(&state, DUMMY_IDENTIFIER_ID, filepath, source_code);
    let hir = LowerToHir::run_all(&state, vec![ast.into()]);

    CollectDefinitions::run_all(&state, &hir);

    assert!(hir
        .first()
        .unwrap()
        .module()
        .symbol_or_panic(&state.db_read_lock(), IdentifierID::from("A"))
        .is_enum());
    assert!(hir
        .first()
        .unwrap()
        .module()
        .symbol_or_panic(&state.db_read_lock(), IdentifierID::from("B"))
        .is_enum());
    assert!(state.diagnostics_read_lock().is_ok());
}

#[test]
fn test_duplicate_definition() {
    let state = State::new();
    let filepath = PathID::from("test.sr");
    let source_code = "enum A {}\nenum A {}";

    let ast = parse_module(&state, DUMMY_IDENTIFIER_ID, filepath, source_code);
    let hir = LowerToHir::run_all(&state, vec![ast.into()]);

    CollectDefinitions::run_all(&state, &hir);

    assert!(state.diagnostics_read_lock().is_fatal());
}

#[test]
fn test_enum_items() {
    let state = State::new();
    let filepath = PathID::from("test.sr");
    let source_code = "enum A { A, B, C }";

    let ast = parse_module(&state, DUMMY_IDENTIFIER_ID, filepath, source_code);
    let hir = LowerToHir::run_all(&state, vec![ast.into()]);

    CollectDefinitions::run_all(&state, &hir);

    let db = state.db_read_lock();

    assert!(hir
        .first()
        .unwrap()
        .module()
        .symbol_or_panic(&db, IdentifierID::from("A"))
        .is_enum());

    let items = hir
        .first()
        .unwrap()
        .module()
        .symbol_or_panic(&db, IdentifierID::from("A"))
        .to_enum_or_panic()
        .items(&db);

    assert!(items.contains_key(&IdentifierID::from("A")));
    assert!(items.contains_key(&IdentifierID::from("B")));
    assert!(items.contains_key(&IdentifierID::from("C")));
}

#[test]
fn duplicate_enum_item_definitions() {
    let state = State::new();
    let filepath = PathID::from("test.sr");
    let source_code = "enum A { A, A }";

    let ast = parse_module(&state, DUMMY_IDENTIFIER_ID, filepath, source_code);
    let hir = LowerToHir::run_all(&state, vec![ast.into()]);

    CollectDefinitions::run_all(&state, &hir);

    assert!(state.diagnostics_read_lock().is_fatal());
}

#[test]
fn test_function() {
    let state = State::new();
    let filepath = PathID::from("test.sr");
    let source_code = "fun a() {}";

    let ast = parse_module(&state, DUMMY_IDENTIFIER_ID, filepath, source_code);
    let hir = LowerToHir::run_all(&state, vec![ast.into()]);

    CollectDefinitions::run_all(&state, &hir);

    assert!(hir
        .first()
        .unwrap()
        .module()
        .symbol_or_panic(&state.db_read_lock(), IdentifierID::from("a"))
        .is_function());
    assert!(state.diagnostics_read_lock().is_ok());
}

#[test]
fn test_struct() {
    let state = State::new();
    let filepath = PathID::from("test.sr");
    let source_code = "struct A {}";

    let ast = parse_module(&state, DUMMY_IDENTIFIER_ID, filepath, source_code);
    let hir = LowerToHir::run_all(&state, vec![ast.into()]);

    CollectDefinitions::run_all(&state, &hir);

    assert!(hir
        .first()
        .unwrap()
        .module()
        .symbol_or_panic(&state.db_read_lock(), IdentifierID::from("A"))
        .is_struct());
    assert!(state.diagnostics_read_lock().is_ok());
}

#[test]
fn test_interface() {
    let state = State::new();
    let filepath = PathID::from("test.sr");
    let source_code = "interface A {}";

    let ast = parse_module(&state, DUMMY_IDENTIFIER_ID, filepath, source_code);
    let hir = LowerToHir::run_all(&state, vec![ast.into()]);

    CollectDefinitions::run_all(&state, &hir);

    assert!(hir
        .first()
        .unwrap()
        .module()
        .symbol_or_panic(&state.db_read_lock(), IdentifierID::from("A"))
        .is_interface());
    assert!(state.diagnostics_read_lock().is_ok());
}

#[test]
fn test_type_alias() {
    let state = State::new();
    let filepath = PathID::from("test.sr");
    let source_code = "type A = int8;";

    let ast = parse_module(&state, DUMMY_IDENTIFIER_ID, filepath, source_code);
    let hir = LowerToHir::run_all(&state, vec![ast.into()]);

    CollectDefinitions::run_all(&state, &hir);

    assert!(hir
        .first()
        .unwrap()
        .module()
        .symbol_or_panic(&state.db_read_lock(), IdentifierID::from("A"))
        .is_type_alias());
    assert!(state.diagnostics_read_lock().is_ok());
}
