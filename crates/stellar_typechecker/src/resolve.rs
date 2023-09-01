use std::iter;

use itertools::Itertools;
use stellar_ast::IdentifierAST;
use stellar_database::{ModuleID, State, Symbol};

use crate::diagnostics::{FailedToResolveNameInModule, FailedToResolvePackage};

pub fn resolve_global_path(state: &State, path: &stellar_ast::ImportPath) -> Option<Symbol> {
    let mut identifiers = path.path.identifiers.iter();
    let first_identifier = identifiers.next()?;

    let Some(root_module) = state
        .db_read_lock()
        .package_root_module(first_identifier.id)
    else {
        state.diagnostics_write_lock().add_single_file_diagnostic(
            first_identifier.location.filepath,
            FailedToResolvePackage::new(
                first_identifier.location,
                first_identifier.id.resolve_or_panic(),
            ),
        );

        return None;
    };

    resolve_global_path_by_first_symbol(
        state,
        Symbol::Module(root_module),
        first_identifier,
        identifiers,
    )
}

fn resolve_global_path_by_first_symbol<'a>(
    state: &State,
    symbol: Symbol,
    first_identifier: &'a IdentifierAST,
    identifiers: impl Iterator<Item = &'a IdentifierAST>,
) -> Option<Symbol> {
    iter::once(first_identifier)
        .chain(identifiers)
        .tuple_windows()
        .try_fold(symbol, |symbol, (first_identifier, identifier)| {
            resolve_global_path_segment(state, symbol, first_identifier, identifier)
        })
}

fn resolve_global_path_segment(
    state: &State,
    symbol: Symbol,
    first_identifier: &IdentifierAST,
    identifier: &IdentifierAST,
) -> Option<Symbol> {
    match symbol {
        Symbol::Module(module) => {
            resolve_symbol_in_module_namespace(state, module, first_identifier, identifier)
        }
        _ => None,
    }
}

fn resolve_symbol_in_module_namespace(
    state: &State,
    module: ModuleID,
    first_identifier: &IdentifierAST,
    identifier: &IdentifierAST,
) -> Option<Symbol> {
    if let Some(symbol) = module
        .submodule(&state.db_read_lock(), identifier.id)
        .map(Symbol::Module)
        .or(module.module_item_symbol(&state.db_read_lock(), identifier.id))
    {
        Some(symbol)
    } else {
        state.diagnostics_write_lock().add_single_file_diagnostic(
            identifier.location.filepath,
            FailedToResolveNameInModule::new(
                identifier.id.resolve_or_panic(),
                identifier.location,
                first_identifier.id.resolve_or_panic(),
                first_identifier.location,
            ),
        );

        None
    }
}
