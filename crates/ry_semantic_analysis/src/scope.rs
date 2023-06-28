use ry_ast::typed::Type;
use ry_diagnostics::{scope::ScopeDiagnostic, BuildDiagnostic, CompilerDiagnostic};
use ry_interner::{Interner, Symbol};
use ry_source_file::span::Span;
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
pub struct LocalScope<'parent> {
    /// Symbols in this scope (not the ones contained in the parent scopes).
    symbols: HashMap<Symbol, SymbolData>,

    /// Parent scope.
    parent: Option<&'parent LocalScope<'parent>>,
}

impl<'parent> LocalScope<'parent> {
    #[inline]
    #[must_use]
    pub fn new(parent: Option<&'parent LocalScope<'parent>>) -> Self {
        Self {
            symbols: HashMap::new(),
            parent,
        }
    }

    /// Returns the parent scope.
    #[inline]
    #[must_use]
    pub const fn parent(&self) -> Option<&'parent LocalScope<'parent>> {
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

    /// Adds a symbol to this scope and if an error occurs, returns `None`.
    pub fn add_symbol(&mut self, symbol: Symbol, data: SymbolData) -> Option<()> {
        if self.symbols.contains_key(&symbol) {
            return None;
        }

        self.symbols.insert(symbol, data);
        Some(())
    }

    /// Adds a symbol to this scope and if an error occurs, it will be added into `diagnostics` and
    /// the function returns `None`.
    ///
    /// Interner `interner` here is used to resolve the symbol for building a diagnostic.
    pub fn add_symbol_or_save_diagnostic(
        &mut self,
        symbol: Symbol,
        data: SymbolData,
        interner: &Interner,
        diagnostics: &mut Vec<CompilerDiagnostic>,
    ) -> Option<()> {
        if let Some(SymbolData { span, .. }) = self.symbols.get(&symbol) {
            diagnostics.push(
                ScopeDiagnostic::SymbolOverwritten {
                    symbol: interner
                        .resolve(symbol)
                        .unwrap_or_else(|| panic!("Symbol {symbol} cannot be resolved"))
                        .to_owned(),
                    first_definition_span: *span,
                    second_definition_span: data.span(),
                }
                .build(),
            );

            None
        } else {
            self.symbols.insert(symbol, data);
            Some(())
        }
    }

    /// Returns the symbol data for the given symbol. If the symbol is not in this scope, `None` is returned.
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
