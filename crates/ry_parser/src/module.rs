use std::{fs, io, path};

use crate::{items::ItemsParser, Parse, ParseState};
use ry_ast::Module;
use ry_diagnostics::Diagnostic;
use ry_interner::Interner;

/// Parse a Ry module.
///
/// # Errors
/// Returns an error if the file contents cannot be read.
#[inline]
pub fn parse_module<P>(
    file_path: P,
    diagnostics: &mut Vec<Diagnostic>,
    interner: &mut Interner,
) -> Result<Module, io::Error>
where
    P: AsRef<path::Path>,
{
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
