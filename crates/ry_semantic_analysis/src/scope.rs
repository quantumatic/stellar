use ry_ast::typed::Type;
use ry_interner::Symbol;
use std::{collections::HashMap, sync::Arc};

pub struct Scope<'a> {
    variables: HashMap<Symbol, Arc<Type>>,
    parent: Option<&'a Scope<'a>>,
}

impl<'a> Scope<'a> {
    pub fn new(parent: Option<&'a Scope<'a>>) -> Scope {
        Scope {
            variables: HashMap::new(),
            parent,
        }
    }

    pub fn parent(&self) -> Option<&'a Scope<'a>> {
        self.parent
    }

    pub fn variables(&self) -> &HashMap<Symbol, Arc<Type>> {
        &self.variables
    }

    pub fn add(&mut self, symbol: Symbol, ty: Arc<Type>) {
        self.variables.insert(symbol, ty);
    }

    pub fn lookup(&self, symbol: Symbol) -> Option<Arc<Type>> {
        if let Some(ty) = self.variables.get(&symbol) {
            Some(ty.clone())
        } else if let Some(parent) = self.parent() {
            parent.lookup(symbol)
        } else {
            None
        }
    }
}
