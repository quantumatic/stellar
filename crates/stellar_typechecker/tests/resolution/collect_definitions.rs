use stellar_ast_lowering::LowerToHir;
use stellar_database::State;
use stellar_interner::{IdentifierID, PathID, DUMMY_IDENTIFIER_ID};
use stellar_parser::parse_module;
use stellar_typechecker::resolution::collect_definitions::CollectDefinitions;

#[test]
fn test_enum() {
    let mut state = State::new();
    let filepath = PathID::from("test.sr");
    let source_code = "enum A {}\nenum B {}";

    let ast = parse_module(&mut state, DUMMY_IDENTIFIER_ID, filepath, source_code);
    let hir = LowerToHir::run_all(&mut state, vec![ast.into()]);

    CollectDefinitions::run_all(&mut state, &hir);

    assert!(hir
        .first_key_value()
        .unwrap()
        .0
        .symbol(state.db(), IdentifierID::from("A"))
        .is_enum());
    assert!(hir
        .first_key_value()
        .unwrap()
        .0
        .symbol(state.db(), IdentifierID::from("B"))
        .is_enum());
    assert!(state.diagnostics().is_ok());
}

#[test]
fn test_duplicate_definition() {
    let mut state = State::new();
    let filepath = PathID::from("test.sr");
    let source_code = "enum A {}\nenum A {}";

    let ast = parse_module(&mut state, DUMMY_IDENTIFIER_ID, filepath, source_code);
    let hir = LowerToHir::run_all(&mut state, vec![ast.into()]);

    CollectDefinitions::run_all(&mut state, &hir);

    assert!(state.diagnostics().is_fatal());
}

#[test]
fn test_enum_items() {
    let mut state = State::new();
    let filepath = PathID::from("test.sr");
    let source_code = "enum A { A, B, C }";

    let ast = parse_module(&mut state, DUMMY_IDENTIFIER_ID, filepath, source_code);
    let hir = LowerToHir::run_all(&mut state, vec![ast.into()]);

    CollectDefinitions::run_all(&mut state, &hir);

    assert!(hir
        .first_key_value()
        .unwrap()
        .0
        .symbol(state.db(), IdentifierID::from("A"))
        .is_enum());

    let items = hir
        .first_key_value()
        .unwrap()
        .0
        .symbol(state.db(), IdentifierID::from("A"))
        .to_enum()
        .items(state.db());

    assert!(items.contains_key(&IdentifierID::from("A")));
    assert!(items.contains_key(&IdentifierID::from("B")));
    assert!(items.contains_key(&IdentifierID::from("C")));
}

#[test]
fn duplicate_enum_item_definitions() {
    let mut state = State::new();
    let filepath = PathID::from("test.sr");
    let source_code = "enum A { A, A }";

    let ast = parse_module(&mut state, DUMMY_IDENTIFIER_ID, filepath, source_code);
    let hir = LowerToHir::run_all(&mut state, vec![ast.into()]);

    CollectDefinitions::run_all(&mut state, &hir);

    assert!(state.diagnostics().is_fatal());
}

#[test]
fn test_function() {
    let mut state = State::new();
    let filepath = PathID::from("test.sr");
    let source_code = "fun a() {}";

    let ast = parse_module(&mut state, DUMMY_IDENTIFIER_ID, filepath, source_code);
    let hir = LowerToHir::run_all(&mut state, vec![ast.into()]);

    CollectDefinitions::run_all(&mut state, &hir);

    assert!(hir
        .first_key_value()
        .unwrap()
        .0
        .symbol(state.db(), IdentifierID::from("a"))
        .is_function());
    assert!(state.diagnostics().is_ok());
}

#[test]
fn test_struct() {
    let mut state = State::new();
    let filepath = PathID::from("test.sr");
    let source_code = "struct A {}";

    let ast = parse_module(&mut state, DUMMY_IDENTIFIER_ID, filepath, source_code);
    let hir = LowerToHir::run_all(&mut state, vec![ast.into()]);

    CollectDefinitions::run_all(&mut state, &hir);

    assert!(hir
        .first_key_value()
        .unwrap()
        .0
        .symbol(state.db(), IdentifierID::from("A"))
        .is_struct());
    assert!(state.diagnostics().is_ok());
}

#[test]
fn test_interface() {
    let mut state = State::new();
    let filepath = PathID::from("test.sr");
    let source_code = "interface A {}";

    let ast = parse_module(&mut state, DUMMY_IDENTIFIER_ID, filepath, source_code);
    let hir = LowerToHir::run_all(&mut state, vec![ast.into()]);

    CollectDefinitions::run_all(&mut state, &hir);

    assert!(hir
        .first_key_value()
        .unwrap()
        .0
        .symbol(state.db(), IdentifierID::from("A"))
        .is_interface());
    assert!(state.diagnostics().is_ok());
}

#[test]
fn test_type_alias() {
    let mut state = State::new();
    let filepath = PathID::from("test.sr");
    let source_code = "type A = int8;";

    let ast = parse_module(&mut state, DUMMY_IDENTIFIER_ID, filepath, source_code);
    let hir = LowerToHir::run_all(&mut state, vec![ast.into()]);

    CollectDefinitions::run_all(&mut state, &hir);

    assert!(hir
        .first_key_value()
        .unwrap()
        .0
        .symbol(state.db(), IdentifierID::from("A"))
        .is_type_alias());
    assert!(state.diagnostics().is_ok());
}
