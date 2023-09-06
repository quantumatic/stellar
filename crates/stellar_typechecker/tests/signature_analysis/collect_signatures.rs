use stellar_ast_lowering::LowerToHir;
use stellar_database::State;
use stellar_interner::{IdentifierID, PathID, DUMMY_IDENTIFIER_ID};
use stellar_parser::parse_module;
use stellar_typechecker::{
    resolution::collect_definitions::CollectDefinitions,
    signature_analysis::collect_signatures::CollectSignatures,
};

#[test]
fn simple_generic_parameter() {
    let mut state = State::new();
    let filepath = PathID::from("test.sr");
    let source_code = "struct Box[T](T);";

    let ast = parse_module(&mut state, DUMMY_IDENTIFIER_ID, filepath, source_code);
    let hir = LowerToHir::run_all(&mut state, vec![ast.into()]);

    CollectDefinitions::run_all(&mut state, &hir);
    CollectSignatures::run_all(&mut state, &hir);

    assert!(hir
        .first()
        .unwrap()
        .module()
        .symbol(state.db(), IdentifierID::from("Box"))
        .to_tuple_like_struct()
        .signature(state.db())
        .generic_parameter_scope(state.db())
        .contains(state.db(), IdentifierID::from("T")));

    assert!(state.diagnostics().is_ok());
}
