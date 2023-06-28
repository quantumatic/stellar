use crate::{items::ItemsParser, Parse, ParseState};
use ry_ast::Module;
use ry_diagnostics::CompilerDiagnostic;
use ry_interner::Interner;
use ry_source_file::{source_file::SourceFile, workspace::FileID};

/// Parse a Ry module.
#[inline]
#[must_use]
pub fn parse_module<'source>(
    file_id: FileID,
    source_file: &'source SourceFile<'source>,
    diagnostics: &mut Vec<CompilerDiagnostic>,
    interner: &mut Interner,
) -> Module<'source> {
    let state = ParseState::new(file_id, source_file, diagnostics, interner);
    parse_module_using(state)
}

/// Parse a Ry module using a given parse state.
#[must_use]
pub fn parse_module_using<'source>(mut state: ParseState<'source, '_, '_>) -> Module<'source> {
    let (global_docstring, first_docstring) = state.consume_module_and_first_item_docstrings();

    Module {
        source_file: state.source_file,
        file_id: state.file_id,
        docstring: global_docstring,
        items: ItemsParser { first_docstring }.parse(&mut state),
    }
}
