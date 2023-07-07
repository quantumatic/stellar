use std::{fs, io, path::Path};

use crate::{items::ItemsParser, Parse, ParseState};
use ry_ast::Module;
use ry_diagnostics::RyDiagnostic;
use ry_interner::Interner;

/// Parse a Ry module.
///
/// # Errors
/// Returns an error if:
/// * the path cannot be resolved by [`FileID`]
/// * the file contents cannot be read.
/// See [`InitializeParseStateError`] for more details.
#[inline]
pub fn parse_module(
    file_path: &Path,
    diagnostics: &mut Vec<RyDiagnostic>,
    interner: &mut Interner,
) -> Result<Module, io::Error> {
    Ok(parse_module_using(ParseState::new(
        &fs::read_to_string(file_path)?,
        diagnostics,
        interner,
    )))
}

/// Parse a Ry module using a given parse state.
#[must_use]
pub fn parse_module_using(mut state: ParseState<'_, '_, '_>) -> Module {
    Module {
        docstring: state.consume_module_docstring(),
        items: ItemsParser.parse(&mut state),
    }
}
