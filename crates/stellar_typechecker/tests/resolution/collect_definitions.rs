use stellar_ast_lowering::LowerToHir;
use stellar_database::{PackageData, State};
use stellar_interner::{IdentifierId, PathId, DUMMY_IDENTIFIER_ID, DUMMY_PATH_ID};
use stellar_parser::parse_module;
use stellar_typechecker::resolution::collect_definitions::CollectDefinitions;

#[test]
fn test_enum() {
    let mut state = State::new();
    let filepath = PathId::from("test.sr");
    let source_code = "enum A {}\nenum B {}";

    let package = PackageData::alloc(state.db_mut(), DUMMY_IDENTIFIER_ID, DUMMY_PATH_ID);
    let parse_result = parse_module(
        &mut state,
        package,
        DUMMY_IDENTIFIER_ID,
        filepath,
        source_code,
    );
    let module = parse_result.module();
    package.set_root_module(state.db_mut(), parse_result.module());

    let hir = LowerToHir::run_all(&mut state, vec![parse_result]);

    CollectDefinitions::run_all(&mut state, &hir);

    assert!(module.symbol(state.db(), IdentifierId::from("A")).is_enum());
    assert!(module.symbol(state.db(), IdentifierId::from("B")).is_enum());
    assert!(state.diagnostics().is_ok());
}

#[test]
fn test_duplicate_definition() {
    let mut state = State::new();
    let filepath = PathId::from("test.sr");
    let source_code = "enum A {}\nenum A {}";

    let package = PackageData::alloc(state.db_mut(), DUMMY_IDENTIFIER_ID, DUMMY_PATH_ID);
    let parse_result = parse_module(
        &mut state,
        package,
        DUMMY_IDENTIFIER_ID,
        filepath,
        source_code,
    );
    package.set_root_module(state.db_mut(), parse_result.module());

    let hir = LowerToHir::run_all(&mut state, vec![parse_result]);

    CollectDefinitions::run_all(&mut state, &hir);

    assert!(state.diagnostics().is_fatal());
}

#[test]
fn test_enum_items() {
    let mut state = State::new();
    let filepath = PathId::from("test.sr");
    let source_code = "enum A { A, B, C }";

    let package = PackageData::alloc(state.db_mut(), DUMMY_IDENTIFIER_ID, DUMMY_PATH_ID);
    let parse_result = parse_module(
        &mut state,
        package,
        DUMMY_IDENTIFIER_ID,
        filepath,
        source_code,
    );
    let module = parse_result.module();
    package.set_root_module(state.db_mut(), parse_result.module());

    let hir = LowerToHir::run_all(&mut state, vec![parse_result]);

    CollectDefinitions::run_all(&mut state, &hir);

    assert!(module.symbol(state.db(), IdentifierId::from("A")).is_enum());

    let items = module
        .symbol(state.db(), IdentifierId::from("A"))
        .to_enum()
        .items(state.db());

    assert!(items.contains_key(&IdentifierId::from("A")));
    assert!(items.contains_key(&IdentifierId::from("B")));
    assert!(items.contains_key(&IdentifierId::from("C")));
}

#[test]
fn duplicate_enum_item_definitions() {
    let mut state = State::new();
    let filepath = PathId::from("test.sr");
    let source_code = "enum A { A, A }";

    let package = PackageData::alloc(state.db_mut(), DUMMY_IDENTIFIER_ID, DUMMY_PATH_ID);
    let parse_result = parse_module(
        &mut state,
        package,
        DUMMY_IDENTIFIER_ID,
        filepath,
        source_code,
    );
    package.set_root_module(state.db_mut(), parse_result.module());

    let hir = LowerToHir::run_all(&mut state, vec![parse_result]);

    CollectDefinitions::run_all(&mut state, &hir);

    assert!(state.diagnostics().is_fatal());
}

#[test]
fn test_function() {
    let mut state = State::new();
    let filepath = PathId::from("test.sr");
    let source_code = "fun a() {}";

    let package = PackageData::alloc(state.db_mut(), DUMMY_IDENTIFIER_ID, DUMMY_PATH_ID);
    let parse_result = parse_module(
        &mut state,
        package,
        DUMMY_IDENTIFIER_ID,
        filepath,
        source_code,
    );
    let module = parse_result.module();
    package.set_root_module(state.db_mut(), parse_result.module());

    let hir = LowerToHir::run_all(&mut state, vec![parse_result]);

    CollectDefinitions::run_all(&mut state, &hir);

    assert!(module
        .symbol(state.db(), IdentifierId::from("a"))
        .is_function());
    assert!(state.diagnostics().is_ok());
}

#[test]
fn test_struct() {
    let mut state = State::new();
    let filepath = PathId::from("test.sr");
    let source_code = "struct A {}";

    let package = PackageData::alloc(state.db_mut(), DUMMY_IDENTIFIER_ID, DUMMY_PATH_ID);
    let parse_result = parse_module(
        &mut state,
        package,
        DUMMY_IDENTIFIER_ID,
        filepath,
        source_code,
    );
    let module = parse_result.module();
    package.set_root_module(state.db_mut(), parse_result.module());

    let hir = LowerToHir::run_all(&mut state, vec![parse_result]);

    CollectDefinitions::run_all(&mut state, &hir);

    assert!(module
        .symbol(state.db(), IdentifierId::from("A"))
        .is_struct());
    assert!(state.diagnostics().is_ok());
}

#[test]
fn test_interface() {
    let mut state = State::new();
    let filepath = PathId::from("test.sr");
    let source_code = "interface A {}";

    let package = PackageData::alloc(state.db_mut(), DUMMY_IDENTIFIER_ID, DUMMY_PATH_ID);
    let parse_result = parse_module(
        &mut state,
        package,
        DUMMY_IDENTIFIER_ID,
        filepath,
        source_code,
    );
    let module = parse_result.module();
    package.set_root_module(state.db_mut(), parse_result.module());

    let hir = LowerToHir::run_all(&mut state, vec![parse_result]);

    CollectDefinitions::run_all(&mut state, &hir);

    assert!(module
        .symbol(state.db(), IdentifierId::from("A"))
        .is_interface());
    assert!(state.diagnostics().is_ok());
}

#[test]
fn test_type_alias() {
    let mut state = State::new();
    let filepath = PathId::from("test.sr");
    let source_code = "type A = int8;";

    let package = PackageData::alloc(state.db_mut(), DUMMY_IDENTIFIER_ID, DUMMY_PATH_ID);
    let parse_result = parse_module(
        &mut state,
        package,
        DUMMY_IDENTIFIER_ID,
        filepath,
        source_code,
    );
    let module = parse_result.module();
    package.set_root_module(state.db_mut(), parse_result.module());

    let hir = LowerToHir::run_all(&mut state, vec![parse_result]);

    CollectDefinitions::run_all(&mut state, &hir);

    assert!(module
        .symbol(state.db(), IdentifierId::from("A"))
        .is_type_alias());
    assert!(state.diagnostics().is_ok());
}
