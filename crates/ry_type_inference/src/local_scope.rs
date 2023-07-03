//! Defines [`LocalScope`] to work with local scopes in statement blocks.

use ry_ast::typed::Type;
use ry_diagnostics::{scope::ScopeDiagnostic, BuildDiagnostic, CompilerDiagnostic};
use ry_interner::{Interner, Symbol};
use ry_workspace::span::Span;
use std::{collections::HashMap, sync::Arc};

/// Information that compiler has about a particular symbol.
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolData {
    /// Span where the symbol was defined.
    span: Span,

    /// Type of the symbol.
    ty: Arc<Type>,
}

impl SymbolData {
    ///
    #[inline]
    #[must_use]
    pub const fn new(span: Span, ty: Arc<Type>) -> Self {
        Self { span, ty }
    }

    /// Returns the span where the symbol was defined.
    #[inline]
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the type of the symbol.
    #[inline]
    #[must_use]
    pub fn ty(&self) -> Arc<Type> {
        self.ty.clone()
    }
}

/// Represents a local scope (a scope that is not a global).
#[derive(Debug)]
pub struct LocalScope<'local_scope> {
    /// Symbols in this scope (not the ones contained in the parent scopes).
    symbols: HashMap<Symbol, SymbolData>,

    /// Parent scope.
    parent: Option<&'local_scope LocalScope<'local_scope>>,
}

impl<'scope> LocalScope<'scope> {
    /// Creates a new [`LocalScope`] instance.
    #[inline]
    #[must_use]
    pub fn new(parent: Option<&'scope LocalScope<'scope>>) -> Self {
        Self {
            symbols: HashMap::new(),
            parent,
        }
    }

    /// Returns the parent scope.
    #[inline]
    #[must_use]
    pub const fn parent(&self) -> Option<&'scope LocalScope<'scope>> {
        self.parent
    }

    /// Returns symbols in this scope (not the ones contained in the parent scopes).
    #[inline]
    #[must_use]
    pub const fn symbols(&self) -> &HashMap<Symbol, SymbolData> {
        &self.symbols
    }

    /// Returns symbols in this scope and all of its parent scopes.
    #[must_use]
    pub fn all_symbols(&self) -> HashMap<Symbol, SymbolData> {
        let mut symbols = self.symbols.clone();

        if let Some(parent) = self.parent() {
            symbols.extend(parent.all_symbols());
        }

        symbols
    }

    /// Adds a symbol to this scope.
    pub fn add_symbol(&mut self, symbol: Symbol, data: SymbolData) {
        // shadowing
        if self.symbols.contains_key(&symbol) {
            self.symbols.remove(&symbol);
        }

        self.symbols.insert(symbol, data);
    }

    /// Returns the symbol data for the given symbol. If the symbol is not in this scope, `None` is returned.
    #[must_use]
    pub fn lookup(&self, symbol: Symbol) -> Option<&SymbolData> {
        if let data @ Some(..) = self.symbols.get(&symbol) {
            data
        } else if let Some(parent) = self.parent() {
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
        diagnostics: &mut Vec<CompilerDiagnostic>,
    ) -> Option<&SymbolData> {
        if let data @ Some(..) = self.lookup(symbol) {
            data
        } else if let Some(parent) = self.parent() {
            parent.lookup_or_save_diagnostic(symbol, span, interner, diagnostics)
        } else {
            diagnostics.push(
                ScopeDiagnostic::SymbolNotFound {
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
