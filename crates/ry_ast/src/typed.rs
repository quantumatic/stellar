//! Defines [`Type`] for working with types and typed AST nodes.

use ry_interner::{symbols::LIST, Symbol};
use std::sync::Arc;

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Unit,
    Constructor {
        path: Vec<Symbol>,
        generic_arguments: Vec<Arc<Self>>,
    },
    Tuple {
        element_types: Vec<Arc<Self>>,
    },
    Function {
        parameter_types: Vec<Arc<Self>>,
        return_type: Arc<Self>,
    },
    Variable(usize),
}

#[inline]
#[must_use]
pub fn primitive_constructor(symbol: Symbol) -> Type {
    Type::Constructor {
        path: vec![symbol],
        generic_arguments: vec![],
    }
}

#[inline]
#[must_use]
pub fn list_of(element_type: Arc<Type>) -> Type {
    Type::Constructor {
        path: vec![LIST],
        generic_arguments: vec![element_type],
    }
}
