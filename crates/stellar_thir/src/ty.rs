//! Defines [`Type`] for working with types and THIR nodes.

use std::fmt::Display;

use derive_more::Display;
use stellar_filesystem::location::Location;
use stellar_interner::{builtin_identifiers, IdentifierID};

use crate::Path;

/// A raw representation of types in the Stellar programming language.
///
/// Compared to [`stellar_hir::Type`] and [`stellar_ast::Type`], doesn't store
/// information about locations and all pathes are fully unwraped.
/// For example: `Iterator` is converted into `std.iterator.Iterator`.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind"))]
pub enum Type {
    /// A unit type: `()`.
    #[cfg_attr(feature = "serde", serde(rename = "unit_type"))]
    Unit,

    /// A type constructor: `List[uint32]`, `uint32`, `String`.
    ///
    /// Anything that has name and optionally have generic arguments.
    #[cfg_attr(feature = "serde", serde(rename = "constructor_type"))]
    Constructor(TypeConstructor),

    /// A tuple type: `(uint32,)`, `(String, uint32)`.
    ///
    /// **Note**: `element_types` vector is never empty, because an
    /// enum variant for unit type already exists: [`Type::Unit`].
    #[cfg_attr(feature = "serde", serde(rename = "tuple_type"))]
    Tuple {
        /// Types of tuple elements.
        element_types: Vec<Self>,
    },

    /// A function type: `(String): bool`, `(): ()`, `(T, M): ()`.
    #[cfg_attr(feature = "serde", serde(rename = "function_type"))]
    Function {
        /// List of function parameter types.
        parameter_types: Vec<Self>,

        /// Return type.
        ///
        /// **Note**: return type is not optional! If function doesn't
        /// return anything, the return type value is [`Type::Unit`].
        return_type: Box<Self>,
    },

    /// A type variable (placeholder for types, that aren't inferred yet).
    #[cfg_attr(feature = "serde", serde(rename = "type_variable"))]
    Variable(TypeVariable),

    /// An interface object type, e.g. `dyn Iterator[char] + ToString`.
    ///
    /// A type of dynamically dispatched objects, that have a vtable of interfaces in
    /// `bounds`.
    #[cfg_attr(feature = "serde", serde(rename = "interface_object_type"))]
    InterfaceObject {
        /// A list of interfaces, that will be used to construct a vtable.
        bounds: Vec<TypeConstructor>,
    },
}

impl Display for Type {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind().fmt(f)
    }
}

/// A kind of type.
///
/// See [`Type`] for more details.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Display)]
pub enum TypeKind {
    /// A unit type, e.g. `()`.
    #[display(fmt = "unit type")]
    Unit,

    /// A type constructor: `List[uint32]`, `uint32`, `String`.
    ///
    /// Anything that has name and optionally have generic arguments.
    #[display(fmt = "type constructor")]
    Constructor,

    /// A tuple type: `(uint32,)`, `(String, uint32)`.
    ///
    /// **Note**: `element_types` vector is never empty, because an
    /// enum variant for unit type already exists: [`Type::Unit`].
    #[display(fmt = "tuple type")]
    Tuple,

    /// A function type: `(String): bool`, `(): ()`, `(T, M): ()`.
    #[display(fmt = "function type")]
    Function,

    /// A type variable (placeholder for types, that aren't inferred yet).
    #[display(fmt = "uninferred type")]
    Variable,

    /// An interface object type, e.g. `dyn Iterator[char] + ToString`.
    ///
    /// A type of dynamically dispatched objects, that have a vtable of interfaces in
    /// `bounds`.
    #[display(fmt = "interface object type")]
    InterfaceObject,
}

impl Type {
    /// Returns a type's kind.
    ///
    /// See [`TypeKind`] for more details.
    #[inline(always)]
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

///
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind"))]
pub enum TypeVariable {
    #[cfg_attr(feature = "serde", serde(rename = "type_argument_variable"))]
    TypePlaceholder {
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
    #[inline(always)]
    #[must_use]
    pub const fn id(&self) -> TypeVariableID {
        match self {
            Self::TypePlaceholder { id, .. } | Self::Expression { id, .. } => *id,
        }
    }
}

/// A type variable ID.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct TypeVariableID(pub usize);

/// The macro, automates the process of generating functions
/// for getting builtin primitive types.
macro_rules! generate_builtin_primitive_types {
    ($($name:ident => $symbol:ident),*) => {
        $(
            #[inline(always)]
            #[must_use]
            #[doc = concat!("Returns a `", stringify!($name), "` type.")]
            pub fn $name() -> Type {
                Type::new_primitive(builtin_identifiers::$symbol)
            }
        )*
    };
}

generate_builtin_primitive_types! {
    int8 => INT8, int16 => INT16, int32 => INT32, int64 => INT64,
    uint8 => UINT8, uint16 => UINT16, uint32 => UINT32, uint64 => UINT64,
    float32 => FLOAT32, float64 => FLOAT64,

    char => CHAR, string => STRING
}

/// A type constructor: `List[uint32]`, `uint32`, `String`.
///
/// Anything that has name and optionally have generic arguments.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct TypeConstructor {
    pub path: Path,
    pub arguments: Vec<Type>,
}

impl TypeConstructor {
    #[inline(always)]
    #[must_use]
    pub const fn new(left: Path, right: Vec<Type>) -> Self {
        Self {
            path: left,
            arguments: right,
        }
    }

    #[inline(always)]
    #[must_use]
    pub fn new_primitive(identifier_id: IdentifierID) -> Self {
        Self {
            path: Path {
                identifiers: vec![identifier_id],
            },
            arguments: vec![],
        }
    }
}

impl Type {
    #[inline(always)]
    #[must_use]
    pub fn new_primitive(identifier_id: IdentifierID) -> Self {
        Self::Constructor(TypeConstructor::new_primitive(identifier_id))
    }
}

/// Returns a list type with the given element type.
#[inline(always)]
#[must_use]
pub fn list_of(element_type: Type) -> Type {
    Type::Constructor(TypeConstructor {
        path: Path {
            identifiers: vec![builtin_identifiers::LIST],
        },
        arguments: vec![element_type],
    })
}
