//! Defines [`Scope`] to work with scopes in statement blocks.

use ry_ast::typed::Type;
use ry_diagnostics::{BuildDiagnostic, RyDiagnostic};
use ry_filesystem::span::Span;
use ry_interner::{Interner, Symbol};
use std::{collections::HashMap, sync::Arc};

use crate::diagnostics::ScopeDiagnostic;

/// Information that compiler has about a particular symbol.
#[derive(Debug, Clone, PartialEq)]
pub struct ValueConstructor {
    /// Span where the symbol was defined.
    pub origin: Span,

    /// Type of the symbol.
    pub ty: Arc<Type>,
}

/// Represents a local scope (a scope that is not a global).
#[derive(Debug)]
pub struct Scope<'scope> {
    /// Symbols in this scope (not the ones contained in the parent scopes).
    entities: HashMap<Symbol, ValueConstructor>,

    /// Parent scope.
    pub parent: Option<&'scope Scope<'scope>>,
}

impl<'scope> Scope<'scope> {
    /// Creates a new [`LocalScope`] instance.
    #[inline]
    #[must_use]
    pub fn new(parent: Option<&'scope Scope<'scope>>) -> Self {
        Self {
            entities: HashMap::new(),
            parent,
        }
    }

    /// Adds a symbol to this scope.
    pub fn add_symbol(&mut self, symbol: Symbol, data: ValueConstructor) {
        // shadowing
        if self.entities.contains_key(&symbol) {
            self.entities.remove(&symbol);
        }

        self.entities.insert(symbol, data);
    }

    /// Returns the symbol data for the given symbol. If the symbol is not in this scope, `None` is returned.
    #[must_use]
    pub fn lookup(&self, symbol: Symbol) -> Option<&ValueConstructor> {
        if let data @ Some(..) = self.entities.get(&symbol) {
            data
        } else if let Some(parent) = self.parent {
            parent.lookup(symbol)
        } else {
            None
        }
    }

    /// Returns the symbol data for the given symbol. If the symbol is not in this scope, `None` is returned
    /// and the error will be added into `diagnostics`.
    ///
    /// Interner `interner` here is used to resolve the symbol for building a diagnostic.
    ///
    /// # Panics
    ///
    /// This function panics if the symbol is not interned in the `interner`.
    pub fn lookup_or_save_diagnostic(
        &self,
        symbol: Symbol,
        span: Span,
        interner: &Interner,
        diagnostics: &mut Vec<RyDiagnostic>,
    ) -> Option<&ValueConstructor> {
        if let data @ Some(..) = self.lookup(symbol) {
            data
        } else if let Some(parent) = self.parent {
            parent.lookup_or_save_diagnostic(symbol, span, interner, diagnostics)
        } else {
            diagnostics.push(
                ScopeDiagnostic::NotFound {
                    symbol: interner
                        .resolve(symbol)
                        .unwrap_or_else(|| panic!("Symbol {symbol} cannot be resolved"))
                        .to_owned(),
                    span,
                }
                .build(),
            );
            None
        }
    }
}
