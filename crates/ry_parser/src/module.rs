use crate::{items::ItemsParser, Parse, ParseState};
use ry_ast::Module;
use ry_diagnostics::CompilerDiagnostic;
use ry_interner::Interner;
use ry_span::{file::InMemoryFile, storage::FileID};

/// Parse a Ry module.
#[inline]
#[must_use]
pub fn parse_module<'storage>(
    file_id: FileID,
    file: &'storage InMemoryFile<'storage>,
    diagnostics: &mut Vec<CompilerDiagnostic>,
    interner: &mut Interner,
) -> Module<'storage> {
    let state = ParseState::new(file_id, file, diagnostics, interner);
    parse_module_using(state)
}

/// Parse a Ry module using a given parse state.
#[must_use]
pub fn parse_module_using<'storage>(mut state: ParseState<'storage, '_, '_>) -> Module<'storage> {
    Module {
        file: state.file,
        file_id: state.file_id,
        docstring: state.consume_module_docstring(),
        items: ItemsParser.parse(&mut state),
    }
}
