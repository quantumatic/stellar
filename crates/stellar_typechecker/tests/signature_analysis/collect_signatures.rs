use stellar_ast_lowering::LowerToHir;
use stellar_database::{PackageData, State};
use stellar_interner::{IdentifierId, DUMMY_IDENTIFIER_ID, DUMMY_PATH_ID};
use stellar_parser::parse_module;
use stellar_typechecker::{
    resolution::collect_definitions::CollectDefinitions,
    signature_analysis::collect_signatures::CollectSignatures,
};

#[test]
fn simple_generic_parameter() {
    let mut state = State::new();
    let source_code = "struct Box[T](T);";

    let package = PackageData::alloc(state.db_mut(), DUMMY_IDENTIFIER_ID, DUMMY_PATH_ID);
    let parse_result = parse_module(
        &mut state,
        package,
        DUMMY_IDENTIFIER_ID,
        DUMMY_PATH_ID,
        source_code,
    );
    let module = parse_result.module();
    let hir = LowerToHir::run_all(&mut state, vec![parse_result]);

    CollectDefinitions::run_all(&mut state, &hir);
    CollectSignatures::run_all(&mut state, &hir);

    assert!(module
        .symbol(state.db(), IdentifierId::from("Box"))
        .to_tuple_like_struct()
        .signature(state.db())
        .generic_parameter_scope(state.db())
        .contains(state.db(), IdentifierId::from("T")));

    assert!(state.diagnostics().is_ok());
}
