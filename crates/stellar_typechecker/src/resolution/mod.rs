pub mod collect_definitions;
pub mod resolve_imports;

use std::iter;

use itertools::Itertools;
use stellar_ast::IdentifierAST;
use stellar_database::{EnumId, ModuleId, PackageId, State, Symbol, TypeAliasId};

use crate::diagnostics::{
    EnumItemsDoNotServeAsNamespaces, FailedToResolveEnumItem, FailedToResolveNameInModule,
    FailedToResolvePackage, ModuleItemsExceptEnumsDoNotServeAsNamespaces,
};

pub(crate) fn resolve_global_path_in_module_context(
    state: &mut State,
    path: &stellar_ast::Path,
    module: ModuleId,
) -> Option<Symbol> {
    let mut identifiers = path.identifiers.iter();
    let namespace = identifiers.next()?;

    let Some(namespace_symbol) = module.symbol_or_none(state.db(), namespace.id) else {
        state
            .diagnostics_mut()
            .add_diagnostic(FailedToResolvePackage::new(
                namespace.location,
                namespace.id,
            ));

        return None;
    };

    resolve_global_path_by_first_symbol(state, namespace_symbol, namespace, identifiers)
}

pub(crate) fn resolve_global_path(
    state: &mut State,
    package: PackageId,
    path: &stellar_ast::ImportPath,
) -> Option<Symbol> {
    let mut identifiers = path.path.identifiers.iter();
    let namespace = identifiers.next()?;

    let Some(package) = (if namespace.id == package.name(state.db()) {
        Some(package)
    } else {
        package.dependencies(state.db()).get(&namespace.id).copied()
    }) else {
        state
            .diagnostics_mut()
            .add_diagnostic(FailedToResolvePackage::new(
                namespace.location,
                namespace.id,
            ));

        return None;
    };

    let root_module = package.root_module(state.db());

    resolve_global_path_by_first_symbol(state, Symbol::Module(root_module), namespace, identifiers)
}

fn resolve_global_path_by_first_symbol<'a>(
    state: &mut State,
    symbol: Symbol,
    namespace: &'a IdentifierAST,
    identifiers: impl Iterator<Item = &'a IdentifierAST>,
) -> Option<Symbol> {
    iter::once(namespace)
        .chain(identifiers)
        .tuple_windows()
        .try_fold(symbol, |symbol, (namespace, member)| {
            resolve_global_path_segment(state, symbol, *namespace, *member)
        })
}

fn resolve_global_path_segment(
    state: &mut State,
    symbol: Symbol,
    namespace: IdentifierAST,
    member: IdentifierAST,
) -> Option<Symbol> {
    match symbol {
        Symbol::Module(module) => {
            resolve_symbol_in_module_namespace(state, module, namespace, member)
        }
        Symbol::Enum(enum_) => resolve_symbol_in_enum_namespace(state, enum_, namespace, member),
        Symbol::EnumItem(_) => {
            state
                .diagnostics_mut()
                .add_diagnostic(EnumItemsDoNotServeAsNamespaces::new(namespace, member));

            None
        }
        _ => {
            state.diagnostics_mut().add_diagnostic(
                ModuleItemsExceptEnumsDoNotServeAsNamespaces::new(
                    namespace,
                    symbol.module_item_kind(),
                    member,
                ),
            );

            None
        }
    }
}

fn resolve_symbol_in_module_namespace(
    state: &mut State,
    module: ModuleId,
    namespace: IdentifierAST,
    member: IdentifierAST,
) -> Option<Symbol> {
    if let Some(symbol) = module
        .submodule(state.db(), member.id)
        .map(Symbol::Module)
        .or(module.module_item_symbol_or_none(state.db(), member.id))
    {
        Some(symbol)
    } else {
        state
            .diagnostics_mut()
            .add_diagnostic(FailedToResolveNameInModule::new(
                member.id,
                member.location,
                namespace.id,
                namespace.location,
            ));

        None
    }
}

fn resolve_symbol_in_enum_namespace(
    state: &mut State,
    enum_: EnumId,
    namespace: IdentifierAST,
    member: IdentifierAST,
) -> Option<Symbol> {
    if let Some(symbol) = enum_.item(state.db(), member.id) {
        Some(Symbol::EnumItem(symbol))
    } else {
        state
            .diagnostics_mut()
            .add_diagnostic(FailedToResolveEnumItem::new(namespace, member));

        None
    }
}
