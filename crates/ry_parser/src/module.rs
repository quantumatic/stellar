use crate::{items::ItemsParser, Parse, ParseState};
use ry_ast::Module;
use ry_diagnostics::CompilerDiagnostic;
use ry_interner::Interner;

/// Parse a Ry module.
pub fn parse_module<'a>(
    state: &'a mut ParseState<'a>,
) -> (Module<'a>, &'a Vec<CompilerDiagnostic>, &'a Interner) {
    let (global_docstring, first_docstring) = state.consume_module_and_first_item_docstrings();

    (
        Module {
            path: state.source_file.path(),
            file_id: state.file_id,
            docstring: global_docstring,
            items: ItemsParser { first_docstring }.parse(state),
        },
        &state.diagnostics,
        state.interner(),
    )
}
