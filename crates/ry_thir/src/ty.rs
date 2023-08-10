//! Defines [`Type`] for working with types and THIR nodes.

use ry_filesystem::location::Location;
use ry_interner::{builtin_symbols, IdentifierID};
use ry_name_resolution::Path;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Type {
    Unit,
    Constructor(TypeConstructor),
    Tuple {
        element_types: Vec<Self>,
    },
    Function {
        parameter_types: Vec<Self>,
        return_type: Box<Self>,
    },
    Variable(TypeVariable),
    InterfaceObject {
        bounds: Vec<TypeConstructor>,
    },
    WithQualifiedPath {
        left: Box<Self>,
        right: TypeConstructor,
        segments: Vec<TypeConstructor>,
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum TypeKind {
    Unit,
    Constructor,
    Tuple,
    Function,
    Variable,
    TraitObject,
    WithQualifiedPath,
}

impl Type {
    #[inline]
    #[must_use]
    pub const fn kind(&self) -> TypeKind {
        match self {
            Self::Constructor(..) => TypeKind::Constructor,
            Self::Tuple { .. } => TypeKind::Tuple,
            Self::Function { .. } => TypeKind::Function,
            Self::Variable(..) => TypeKind::Variable,
            Self::InterfaceObject { .. } => TypeKind::TraitObject,
            Self::WithQualifiedPath { .. } => TypeKind::WithQualifiedPath,
            Self::Unit => TypeKind::Unit,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum TypeVariable {
    TypeArgument {
        /// Interned name of the corresponding generic parameter.
        symbol: IdentifierID,
        /// Location of the type argument itself (if exists), e.g. location `_` in `HashMap[_, int32]`.
        location: Option<Location>,
        /// Path to the type that contains the correspoding generic parameter.
        origin_type_path: Path,
        /// Location of the corresponding generic parameter name.
        origin_location: Location,
        id: TypeVariableID,
    },
    InvalidType {
        location: Location,
        id: TypeVariableID,
    },
    Expression {
        location: Location,
        id: TypeVariableID,
    },
}

impl TypeVariable {
    #[inline]
    #[must_use]
    pub const fn id(&self) -> TypeVariableID {
        match self {
            Self::TypeArgument { id, .. }
            | Self::InvalidType { id, .. }
            | Self::Expression { id, .. } => *id,
        }
    }
}

pub type TypeVariableID = usize;

pub trait Typed {
    #[must_use]
    fn ty(&self) -> Type;
}

/// Creates a type constructor for a given symbol.
#[inline]
#[must_use]
fn primitive_constructor(symbol: IdentifierID) -> Type {
    Type::Constructor(TypeConstructor {
        left: Path {
            symbols: vec![symbol],
        },
        right: vec![],
    })
}

macro_rules! t {
    ($name:ident, $symbol:ident) => {
        #[inline]
        #[must_use]
        #[doc = concat!("Returns a `", stringify!($name), "` type.")]
        pub fn $name() -> Type {
            primitive_constructor(builtin_symbols::$symbol)
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

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct TypeConstructor {
    pub left: Path,
    pub right: Vec<Type>,
}

/// Returns a list type with the given element type.
#[inline]
#[must_use]
pub fn list_of(element_type: Type) -> Type {
    Type::Constructor(TypeConstructor {
        left: Path {
            symbols: vec![builtin_symbols::LIST],
        },
        right: vec![element_type],
    })
}
