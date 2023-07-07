use std::{fs, io};

use crate::{items::ItemsParser, Parse, ParseState};
use ry_ast::Module;
use ry_diagnostics::FileDiagnostic;
use ry_filesystem::path_resolver::{FileID, FilePathResolver};
use ry_interner::Interner;

/// Error occurs when trying to initialize parse state, but something
/// goes wrong.
#[derive(Debug)]
pub enum InitializeParseStateError {
    /// When the path cannot be resolved by [`FileID`].
    ResolvePathError(FileID),

    /// When the file contents cannot be read.
    ReadFileError(io::Error),
}

/// Parse a Ry module or panic.
///
/// # Panics
/// Panics in the same situations [`parse_module()`] does.
#[inline]
#[must_use]
pub fn parse_module_or_panic(
    file_id: FileID,
    path_resolver: &FilePathResolver<'_>,
    diagnostics: &mut Vec<FileDiagnostic>,
    interner: &mut Interner,
) -> Module {
    parse_module(file_id, path_resolver, diagnostics, interner)
        .unwrap_or_else(|err| panic!("failed to parse file {}: {:?}", file_id, err))
}

/// Parse a Ry module.
///
/// # Errors
/// Returns an error if:
/// * the path cannot be resolved by [`FileID`]
/// * the file contents cannot be read.
/// See [`InitializeParseStateError`] for more details.
#[inline]
pub fn parse_module(
    file_id: FileID,
    path_resolver: &FilePathResolver<'_>,
    diagnostics: &mut Vec<FileDiagnostic>,
    interner: &mut Interner,
) -> Result<Module, InitializeParseStateError> {
    let Some(file_path) = path_resolver.resolve_path(file_id) else {
        return Err(InitializeParseStateError::ResolvePathError(file_id));
    };

    let source = match fs::read_to_string(file_path) {
        Ok(source) => source,
        Err(err) => return Err(InitializeParseStateError::ReadFileError(err)),
    };

    let state = ParseState::new(file_id, &source, diagnostics, interner);

    Ok(parse_module_using(state))
}

/// Parse a Ry module using a given parse state.
#[must_use]
pub fn parse_module_using(mut state: ParseState<'_, '_, '_>) -> Module {
    Module {
        file_id: state.file_id,
        docstring: state.consume_module_docstring(),
        items: ItemsParser.parse(&mut state),
    }
}
