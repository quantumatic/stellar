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

pub fn primitive_constructor(symbol: Symbol) -> Type {
    Type::Constructor {
        path: vec![symbol],
        generic_arguments: vec![],
    }
}

pub fn list(element_type: Arc<Type>) -> Type {
    Type::Constructor {
        path: vec![LIST],
        generic_arguments: vec![element_type],
    }
}
