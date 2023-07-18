//! Defines [`Type`] for working with types and THIR nodes.

use std::sync::Arc;

use ry_interner::{symbols, Symbol};

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Unit,
    Constructor {
        path: TypePath,
    },
    Tuple {
        element_types: Vec<Arc<Self>>,
    },
    Function {
        parameter_types: Vec<Arc<Self>>,
        return_type: Arc<Self>,
    },
    Variable(TypeVariableID),
    Placeholder,
    TraitObject {
        bounds: Vec<TypePath>,
    },
}

pub type TypeVariableID = usize;

pub trait Typed {
    #[must_use]
    fn ty(&self) -> Type;
}

/// Creates a type constructor for a given symbol.
#[inline]
#[must_use]
fn primitive_constructor(symbol: Symbol) -> Type {
    Type::Constructor {
        path: TypePath {
            segments: vec![TypePathSegment {
                left: Path {
                    symbols: vec![symbol],
                },
                right: vec![],
            }],
        },
    }
}

macro_rules! t {
    ($name:ident, $symbol:ident) => {
        #[inline]
        #[must_use]
        #[doc = concat!("Returns a `", stringify!($name), "` type.")]
        pub fn $name() -> Type {
            primitive_constructor(symbols::$symbol)
        }
    };
}

t!(int8, INT8);
t!(int16, INT16);
t!(int32, INT32);
t!(int64, INT64);
t!(uint8, UINT8);
t!(uint16, UINT16);
t!(uint32, UINT32);
t!(uint64, UINT64);
t!(float32, FLOAT32);
t!(float64, FLOAT64);
t!(char, CHAR);
t!(string, STRING);

#[derive(Debug, PartialEq, Clone)]
pub struct TypePath {
    pub segments: Vec<TypePathSegment>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypePathSegment {
    pub left: Path,
    pub right: Vec<Type>,
}

#[derive(Debug, PartialEq, Eq, Clone)]

pub struct Path {
    pub symbols: Vec<Symbol>,
}

/// Returns a list type with the given element type.
#[inline]
#[must_use]
pub fn list_of(element_type: Type) -> Type {
    Type::Constructor {
        path: TypePath {
            segments: vec![TypePathSegment {
                left: Path {
                    symbols: vec![symbols::LIST],
                },
                right: vec![element_type],
            }],
        },
    }
}