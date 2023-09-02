use std::iter;

use itertools::Itertools;
use stellar_ast::IdentifierAST;
use stellar_database::{ModuleID, State, Symbol};

use crate::diagnostics::{FailedToResolveNameInModule, FailedToResolvePackage};

pub fn resolve_global_path(state: &mut State, path: &stellar_ast::ImportPath) -> Option<Symbol> {
    let mut identifiers = path.path.identifiers.iter();
    let first_identifier = identifiers.next()?;

    let Some(root_module) = state.db_mut().package_root_module(first_identifier.id) else {
        state.diagnostics_mut().add_single_file_diagnostic(
            first_identifier.location.filepath,
            FailedToResolvePackage::new(first_identifier.location, first_identifier.id),
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
    state: &mut State,
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
    state: &mut State,
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
    state: &mut State,
    module: ModuleID,
    first_identifier: &IdentifierAST,
    identifier: &IdentifierAST,
) -> Option<Symbol> {
    if let Some(symbol) = module
        .submodule(state.db(), identifier.id)
        .map(Symbol::Module)
        .or(module.module_item_symbol(state.db(), identifier.id))
    {
        Some(symbol)
    } else {
        state.diagnostics_mut().add_single_file_diagnostic(
            identifier.location.filepath,
            FailedToResolveNameInModule::new(
                identifier.id,
                identifier.location,
                first_identifier.id,
                first_identifier.location,
            ),
        );

        None
    }
}
