//! Defines [`Type`] for working with types and THIR nodes.

use ry_filesystem::location::Location;
use ry_interner::{builtin_identifiers, IdentifierID};
use ry_name_resolution::Path;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind"))]
pub enum Type {
    #[cfg_attr(feature = "serde", serde(rename = "unit_type"))]
    Unit,
    #[cfg_attr(feature = "serde", serde(rename = "constructor_type"))]
    Constructor(TypeConstructor),
    #[cfg_attr(feature = "serde", serde(rename = "tuple_type"))]
    Tuple { element_types: Vec<Self> },
    #[cfg_attr(feature = "serde", serde(rename = "function_type"))]
    Function {
        parameter_types: Vec<Self>,
        return_type: Box<Self>,
    },
    #[cfg_attr(feature = "serde", serde(rename = "type_variable"))]
    Variable(TypeVariable),
    #[cfg_attr(feature = "serde", serde(rename = "interface_object_type"))]
    InterfaceObject { bounds: Vec<TypeConstructor> },
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum TypeKind {
    Invalid,
    Unit,
    Constructor,
    Tuple,
    Function,
    Variable,
    InterfaceObject,
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
            Self::InterfaceObject { .. } => TypeKind::InterfaceObject,
            Self::Unit => TypeKind::Unit,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind"))]
pub enum TypeVariable {
    #[cfg_attr(feature = "serde", serde(rename = "type_argument_variable"))]
    TypeArgument {
        /// Interned name of the corresponding generic parameter.
        symbol: IdentifierID,

        /// Location of the type argument itself (if exists), e.g. location `_` in `HashMap[_, int32]`.
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        location: Option<Location>,

        /// Path to the type that contains the correspoding generic parameter.
        origin_type_path: Path,
        /// Location of the corresponding generic parameter name.
        origin_location: Location,

        /// Type variable ID.
        id: TypeVariableID,
    },
    #[cfg_attr(feature = "serde", serde(rename = "expression_type_variable"))]
    Expression {
        /// Location of the expression.
        location: Location,

        /// Type variable ID.
        id: TypeVariableID,
    },
}

impl TypeVariable {
    /// Returns ID of the type variable.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> TypeVariableID {
        match self {
            Self::TypeArgument { id, .. } | Self::Expression { id, .. } => *id,
        }
    }
}

/// A type variable ID.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct TypeVariableID(pub usize);

macro_rules! builtin_types {
    ($($name:ident => $symbol:ident),*) => {
        $(
            #[inline]
            #[must_use]
            #[doc = concat!("Returns a `", stringify!($name), "` type.")]
            pub fn $name() -> Type {
                Type::Constructor(TypeConstructor::primitive(builtin_identifiers::$symbol))
            }
        )*
    };
}

builtin_types! {
    int8 => INT8, int16 => INT16, int32 => INT32, int64 => INT64,
    uint8 => UINT8, uint16 => UINT16, uint32 => UINT32, uint64 => UINT64,
    float32 => FLOAT32, float64 => FLOAT64,
    char => CHAR, string => STRING
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct TypeConstructor {
    pub path: Path,
    pub arguments: Vec<Type>,
}

impl TypeConstructor {
    #[inline]
    #[must_use]
    pub const fn new(left: Path, right: Vec<Type>) -> Self {
        Self {
            path: left,
            arguments: right,
        }
    }

    #[inline]
    #[must_use]
    pub fn primitive(identifier_id: IdentifierID) -> Self {
        Self {
            path: Path {
                identifiers: vec![identifier_id],
            },
            arguments: vec![],
        }
    }
}

impl Type {
    #[inline]
    #[must_use]
    pub fn primitive(identifier_id: IdentifierID) -> Self {
        Self::Constructor(TypeConstructor::primitive(identifier_id))
    }

    #[inline]
    #[must_use]
    pub fn list_type(self) -> Self {
        Self::Constructor(TypeConstructor {
            path: Path {
                identifiers: vec![builtin_identifiers::LIST],
            },
            arguments: vec![self],
        })
    }
}

/// Returns a list type with the given element type.
#[inline]
#[must_use]
pub fn list_of(element_type: Type) -> Type {
    Type::Constructor(TypeConstructor {
        path: Path {
            identifiers: vec![builtin_identifiers::LIST],
        },
        arguments: vec![element_type],
    })
}
