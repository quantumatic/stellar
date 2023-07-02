use crate::{items::ItemsParser, Parse, ParseState};
use ry_ast::Module;
use ry_diagnostics::CompilerDiagnostic;
use ry_interner::Interner;
use ry_workspace::{file::SourceFile, workspace::FileID};

/// Parse a Ry module.
#[inline]
#[must_use]
pub fn parse_module<'workspace>(
    file_id: FileID,
    source_file: &'workspace SourceFile<'workspace>,
    diagnostics: &mut Vec<CompilerDiagnostic>,
    interner: &mut Interner,
) -> Module<'workspace> {
    let state = ParseState::new(file_id, source_file, diagnostics, interner);
    parse_module_using(state)
}

/// Parse a Ry module using a given parse state.
#[must_use]
pub fn parse_module_using<'workspace>(
    mut state: ParseState<'workspace, '_, '_>,
) -> Module<'workspace> {
    Module {
        source_file: state.source_file,
        file_id: state.file_id,
        docstring: state.consume_module_docstring(),
        items: ItemsParser.parse(&mut state),
    }
}
