use ry_ast::typed::Type;
use ry_interner::Symbol;
use ry_source_file::span::Span;
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone, PartialEq)]
pub struct SymbolData {
    span: Span,
    ty: Arc<Type>,
}

impl SymbolData {
    #[inline]
    #[must_use]
    pub const fn new(span: Span, ty: Arc<Type>) -> Self {
        Self { span, ty }
    }

    #[inline]
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    #[inline]
    #[must_use]
    pub fn ty(&self) -> Arc<Type> {
        self.ty.clone()
    }
}

#[derive(Debug, Clone)]
pub struct Scope<'a> {
    symbols: HashMap<Symbol, SymbolData>,
    parent: Option<&'a Scope<'a>>,
}

impl<'a> Scope<'a> {
    #[inline]
    #[must_use]
    pub fn new(parent: Option<&'a Scope<'a>>) -> Self {
        Self {
            symbols: HashMap::new(),
            parent,
        }
    }

    #[inline]
    #[must_use]
    pub const fn parent(&self) -> Option<&'a Scope<'a>> {
        self.parent
    }

    #[inline]
    #[must_use]
    pub const fn symbols(&self) -> &HashMap<Symbol, SymbolData> {
        &self.symbols
    }

    pub fn add(&mut self, symbol: Symbol, data: SymbolData) {
        self.symbols.insert(symbol, data);
    }

    pub fn lookup(&self, symbol: Symbol) -> Option<&SymbolData> {
        if let data @ Some(..) = self.symbols.get(&symbol) {
            data
        } else if let Some(parent) = self.parent() {
            parent.lookup(symbol)
        } else {
            None
        }
    }
}
