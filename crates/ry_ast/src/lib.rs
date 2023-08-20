//! # Token
//!
//! Token is a grammatical unit of the Ry programming language. It is defined
//! in the [`token`] module. See [`Token`] and [`RawToken`] for more information.
//!
//! # Abstract Syntax Tree
//!
//! AST (or Abstract Syntax Tree) is a representation of the code that stores
//! information about relations between tokens. It can be emitted by
//! the parser defined in [`ry_parser`] crate.
//!
//! For example, the following code:
//!
//! ```ry
//! fun main() {
//!     println("hello world");
//! }
//! ```
//!
//! Will be converted during parsing stage to AST, which when serialized will look like this:
//!
//! ```json
//! {
//!     "items": [
//!         {
//!             "kind": "function_item",
//!             "signature": {
//!                 "visibility": {
//!                     "kind": "private"
//!                 },
//!                 "name": {
//!                     "location": {
//!                         "file_path_id": 1,
//!                         "start": 4,
//!                         "end": 8
//!                     },
//!                     "id": 21
//!                 },
//!                 "generic_parameters": [],
//!                 "parameters": [],
//!                 "where_predicates": []
//!             },
//!             "body": [
//!                 {
//!                     "kind": "expression_statement",
//!                     "expression": {
//!                         "kind": "call_expression",
//!                         "location": {
//!                             "file_path_id": 1,
//!                             "start": 15,
//!                             "end": 37
//!                         },
//!                         "callee": {
//!                             "kind": "identifier_expression",
//!                             "location": {
//!                                 "file_path_id": 1,
//!                                 "start": 15,
//!                                 "end": 22
//!                             },
//!                             "id": 22
//!                         },
//!                         "arguments": [
//!                             {
//!                                 "kind": "literal_expression",
//!                                 "literal_kind": "string",
//!                                 "value": "hello world",
//!                                 "location": {
//!                                     "file_path_id": 1,
//!                                     "start": 23,
//!                                     "end": 36
//!                                 }
//!                             }
//!                         ]
//!                     },
//!                     "has_semicolon": true
//!                 }
//!             ]
//!         }
//!     ]
//! }
//! ```
//!
//! For more details see the module items and start with [`Module`] node.
//!
//!
//! # Serialization
//!
//! If the `serde` feature is enabled, the AST can be serialized using the `serde`
//! crate.
//!
//! [`Token`]: crate::token::Token
//! [`ry_parser`]: ../ry_parser/index.html

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png",
    html_favicon_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png"
)]
#![warn(clippy::dbg_macro)]
#![warn(
    // rustc lint groups https://doc.rust-lang.org/rustc/lints/groups.html
    future_incompatible,
    let_underscore,
    nonstandard_style,
    rust_2018_compatibility,
    rust_2018_idioms,
    rust_2021_compatibility,
    unused,
    // rustc allowed-by-default lints https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    missing_debug_implementations,
    non_ascii_idents,
    noop_method_call,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_op_in_unsafe_fn,
    //unused_crate_dependencies,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_tuple_struct_fields,
    variant_size_differences,
    // rustdoc lints https://doc.rust-lang.org/rustdoc/lints.html
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    //rustdoc::missing_crate_level_docs,
    rustdoc::private_doc_tests,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::bare_urls,
    // clippy categories https://doc.rust-lang.org/clippy/
    clippy::all,
    clippy::correctness,
    clippy::suspicious,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::pedantic,
    clippy::nursery,
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::too_many_lines,
    clippy::option_if_let_else,
    clippy::unnested_or_patterns
)]

use std::fmt::Display;
#[cfg(feature = "serde")]
use std::str::FromStr;

use derive_more::Display;
use ry_filesystem::location::Location;
use ry_interner::IdentifierID;
#[cfg(feature = "serde")]
use serde::Deserializer;
#[cfg(feature = "serde")]
use serde::Serializer;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use token::{Punctuator, RawToken};

pub mod precedence;
pub mod token;
pub mod visit;

/// A literal, e.g. `true`, `3`, `\"hello\"`.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "literal_kind"))]
pub enum Literal {
    /// Boolean literal, e.g. `true` or `false`.
    #[cfg_attr(feature = "serde", serde(rename = "boolean"))]
    Boolean { value: bool, location: Location },

    /// Character literal, e.g. `'a'`, `'\u{1234}'`.
    #[cfg_attr(feature = "serde", serde(rename = "character"))]
    Character { value: char, location: Location },

    /// String literal, e.g. `"hello"`.
    #[cfg_attr(feature = "serde", serde(rename = "string"))]
    String { value: String, location: Location },

    /// Integer literal, e.g. `123`,
    #[cfg_attr(feature = "serde", serde(rename = "integer"))]
    Integer { value: u64, location: Location },

    /// Float literal, e.g. `3.14`.
    #[cfg_attr(feature = "serde", serde(rename = "float"))]
    Float { value: f64, location: Location },
}

impl Literal {
    #[inline]
    #[must_use]
    pub const fn location(&self) -> Location {
        match self {
            Self::Boolean { location, .. }
            | Self::Character { location, .. }
            | Self::String { location, .. }
            | Self::Integer { location, .. }
            | Self::Float { location, .. } => *location,
        }
    }
}

/// An identifier with a specified location, e.g. `foo`, `std`.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IdentifierAST {
    pub location: Location,
    pub id: IdentifierID,
}

/// A sequence of identifiers separated by `.`, e.g. `std.io`, `foo`.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Path {
    pub location: Location,
    pub identifiers: Vec<IdentifierAST>,
}

/// An import path, e.g. `std.io`, `std.io as myio`.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ImportPath {
    pub path: Path,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub r#as: Option<IdentifierAST>,
}

/// A type constructor, e.g. `Option[T]`.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TypeConstructor {
    pub location: Location,
    pub path: Path,
    pub arguments: Vec<Type>,
}

/// A pattern, e.g. `Some(x)`, `None`, `a @ [3, ..]`, `[1, .., 3]`, `(1, \"hello\")`, `3.2`.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind"))]
pub enum Pattern {
    /// A literal pattern, e.g. `3.14`, `'a'`, `true`.
    #[cfg_attr(feature = "serde", serde(rename = "literal_pattern"))]
    Literal(Literal),

    /// An identifier pattern, e.g. `f`, `list @ [3, ..]`.
    #[cfg_attr(feature = "serde", serde(rename = "identifier_pattern"))]
    Identifier {
        location: Location,
        identifier: IdentifierAST,

        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pattern: Option<Box<Self>>,
    },

    /// A struct pattern, e.g. `Person { name, age, .. }`.
    #[cfg_attr(feature = "serde", serde(rename = "struct_pattern"))]
    Struct {
        location: Location,
        path: Path,
        fields: Vec<StructFieldPattern>,
    },

    /// A tuple-like pattern - used to match a tuple-like structs and enum tuple-like items,
    /// e.g. `Some(x)`, `A()`.
    #[cfg_attr(feature = "serde", serde(rename = "tuple_like_pattern"))]
    TupleLike {
        location: Location,
        path: Path,
        inner_patterns: Vec<Self>,
    },

    /// A tuple pattern, e.g. `(a, "hello", ..)`.
    #[cfg_attr(feature = "serde", serde(rename = "tuple_pattern"))]
    Tuple {
        location: Location,
        elements: Vec<Self>,
    },

    /// A path pattern.
    #[cfg_attr(feature = "serde", serde(rename = "path_pattern"))]
    Path { path: Path },

    /// A list pattern, e.g. `[1, .., 10]`.
    #[cfg_attr(feature = "serde", serde(rename = "list_pattern"))]
    List {
        location: Location,
        inner_patterns: Vec<Self>,
    },

    /// A grouped pattern - surrounded by parentheses, e.g. `(a)`, `([1, .., 9])`.
    #[cfg_attr(feature = "serde", serde(rename = "grouped_pattern"))]
    Grouped {
        location: Location,
        inner: Box<Self>,
    },

    /// An or pattern, e.g. `Some(..) | None`.
    #[cfg_attr(feature = "serde", serde(rename = "or_pattern"))]
    Or {
        location: Location,
        left: Box<Self>,
        right: Box<Self>,
    },

    /// A rest pattern - `..`.
    #[cfg_attr(feature = "serde", serde(rename = "rest_pattern"))]
    Rest { location: Location },
}

impl Pattern {
    /// Returns the location of the pattern.
    #[inline]
    #[must_use]
    pub const fn location(&self) -> Location {
        match self {
            Self::Literal(
                Literal::Boolean { location, .. }
                | Literal::Character { location, .. }
                | Literal::String { location, .. }
                | Literal::Integer { location, .. }
                | Literal::Float { location, .. },
            )
            | Self::Grouped { location, .. }
            | Self::Identifier { location, .. }
            | Self::List { location, .. }
            | Self::Or { location, .. }
            | Self::Rest { location }
            | Self::Struct { location, .. }
            | Self::Tuple { location, .. }
            | Self::TupleLike { location, .. }
            | Self::Path {
                path: Path { location, .. },
            } => *location,
        }
    }
}

/// A pattern used to match a struct field, e.g. `citizenship: "USA"`, `name` and `..` in
/// `Person { citizenship: "USA", name, .. }`
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum StructFieldPattern {
    /// A pattern used to match a struct field, which is not rest pattern (`..`),
    /// e.g. `citizen: "USA"` and `name` in `Person { citizen: "USA", name, .. }`.
    NotRest {
        location: Location,
        field_name: IdentifierAST,

        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        value_pattern: Option<Pattern>,
    },
    /// A rest pattern, e.g. `..`.
    Rest { location: Location },
}

/// A type, e.g. `int32`, `(char): bool`, `(char, char)`.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind"))]
pub enum Type {
    /// A type path, e.g. `char`, `Option[T]`.
    #[cfg_attr(feature = "serde", serde(rename = "type_constructor"))]
    Constructor(TypeConstructor),

    /// A tuple type, e.g. `(int32, String, char)`.
    #[cfg_attr(feature = "serde", serde(rename = "tuple_type"))]
    Tuple {
        location: Location,
        element_types: Vec<Self>,
    },

    /// A function type (return type is required for consistency), e.g. `(char): bool`.
    #[cfg_attr(feature = "serde", serde(rename = "function_type"))]
    Function {
        location: Location,
        parameter_types: Vec<Self>,
        return_type: Box<Self>,
    },

    /// A parenthesized type, e.g. `(int32)`.
    ///
    /// **Note**: parenthesized type is not a single element tuple type, because
    /// its syntax is: `(T,)`!
    #[cfg_attr(feature = "serde", serde(rename = "parenthesized_type"))]
    Parenthesized {
        location: Location,
        inner: Box<Self>,
    },

    /// An interface object type, e.g. `dyn Iterator[Item = uint32]`, `dyn Debug + Clone`.
    #[cfg_attr(feature = "serde", serde(rename = "interface_object_type"))]
    InterfaceObject {
        location: Location,
        bounds: Vec<TypeConstructor>,
    },
}

impl Type {
    /// Returns the location of the type.
    #[inline]
    #[must_use]
    pub const fn location(&self) -> Location {
        match self {
            Self::Function { location, .. }
            | Self::Parenthesized { location, .. }
            | Self::Constructor(TypeConstructor { location, .. })
            | Self::InterfaceObject { location, .. }
            | Self::Tuple { location, .. } => *location,
        }
    }
}

/// A type parameter, e.g. `T` in `fun into[T](a: T);`.
#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GenericParameter {
    pub name: IdentifierAST,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub bounds: Option<Vec<TypeConstructor>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub default_value: Option<Type>,
}

/// A type alias, e.g. `type MyResult = Result[String, MyError];`.
#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TypeAlias {
    pub visibility: Visibility,
    pub name: IdentifierAST,
    pub generic_parameters: Vec<GenericParameter>,
    pub value: Type,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub docstring: Option<String>,
}

/// A where clause predicate, e.g. `T: ToString`.
#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct WherePredicate {
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub ty: Type,

    pub bounds: Vec<TypeConstructor>,
}

/// An expression.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind"))]
pub enum Expression {
    /// List expression, e.g. `[1, 2, 3]`.
    #[cfg_attr(feature = "serde", serde(rename = "list_expression"))]
    List {
        location: Location,
        elements: Vec<Self>,
    },

    /// As expression, e.g. `a as float32`.
    #[cfg_attr(feature = "serde", serde(rename = "as_expression"))]
    As {
        location: Location,
        left: Box<Self>,
        right: Type,
    },

    /// Loop expression, e.g. `loop { ... }`
    #[cfg_attr(feature = "serde", serde(rename = "loop_expression"))]
    Loop {
        location: Location,
        statements_block: Vec<Statement>,
    },

    /// Binary expression, e.g. `1 + 2`.
    #[cfg_attr(feature = "serde", serde(rename = "binary_expression"))]
    Binary {
        location: Location,
        left: Box<Self>,
        operator: BinaryOperator,
        right: Box<Self>,
    },

    /// Block expression, e.g. `{ let b = 1; b }`.
    #[cfg_attr(feature = "serde", serde(rename = "statements_block_expression"))]
    StatementsBlock {
        location: Location,
        block: Vec<Statement>,
    },

    /// Literal expression, e.g. `true`, `\"hello\"`, `1.2`.
    #[cfg_attr(feature = "serde", serde(rename = "literal_expression"))]
    Literal(Literal),

    /// Identifier expression, e.g. `foo`.
    #[cfg_attr(feature = "serde", serde(rename = "identifier_expression"))]
    Identifier(IdentifierAST),

    /// Parenthesized expression, e.g. `(1 + 2)`.
    #[cfg_attr(feature = "serde", serde(rename = "parenthesized_expression"))]
    Parenthesized {
        location: Location,
        inner: Box<Self>,
    },

    /// If expression, e.g. `if x { ... } else { ... }`.
    #[cfg_attr(feature = "serde", serde(rename = "if_expression"))]
    If {
        location: Location,
        if_blocks: Vec<(Self, Vec<Statement>)>,
        r#else: Option<Vec<Statement>>,
    },

    /// Field access expression, e.g. `x.y`.
    #[cfg_attr(feature = "serde", serde(rename = "field_access_expression"))]
    FieldAccess {
        location: Location,
        left: Box<Self>,
        right: IdentifierAST,
    },

    /// Prefix expression, e.g. `!false`, `++a`.
    #[cfg_attr(feature = "serde", serde(rename = "prefix_expression"))]
    Prefix {
        location: Location,
        inner: Box<Self>,
        operator: PrefixOperator,
    },

    /// Postfix expression, e.g. `safe_div(1, 0)?`, `a++`.
    #[cfg_attr(feature = "serde", serde(rename = "postfix_expression"))]
    Postfix {
        location: Location,
        inner: Box<Self>,
        operator: PostfixOperator,
    },

    /// While expression, e.g. `while x != 0 {}`.
    #[cfg_attr(feature = "serde", serde(rename = "while_expression"))]
    While {
        location: Location,
        condition: Box<Self>,
        statements_block: Vec<Statement>,
    },

    /// Call expression, e.g. `s.to_string()`.
    #[cfg_attr(feature = "serde", serde(rename = "call_expression"))]
    Call {
        location: Location,
        callee: Box<Self>,
        arguments: Vec<Self>,
    },

    /// Type arguments expression, e.g. `sizeof[uint32]`.
    #[cfg_attr(feature = "serde", serde(rename = "type_arguments_expression"))]
    TypeArguments {
        location: Location,
        left: Box<Self>,
        arguments: Vec<Type>,
    },

    /// Tuple expression, e.g. `(a, 32, \"hello\")`.
    #[cfg_attr(feature = "serde", serde(rename = "tuple_expression"))]
    Tuple {
        location: Location,
        elements: Vec<Self>,
    },

    /// Struct expression, e.g. `Person { name: \"John\", age: 25 }`.
    #[cfg_attr(feature = "serde", serde(rename = "struct_expression"))]
    Struct {
        location: Location,
        left: Box<Self>,
        fields: Vec<StructFieldExpression>,
    },

    /// Match expression (`match fs.read_file(...) { ... }`).
    #[cfg_attr(feature = "serde", serde(rename = "match_expression"))]
    Match {
        location: Location,
        expression: Box<Self>,
        block: Vec<MatchExpressionItem>,
    },

    /// Lambda expression (`|x| { x + 1 }`).
    #[cfg_attr(feature = "serde", serde(rename = "lambda_expression"))]
    Lambda {
        location: Location,
        parameters: Vec<LambdaFunctionParameter>,

        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        return_type: Option<Type>,

        value: Box<Self>,
    },
}

/// A lambda function parameter, e.g. `x` in `|x| { x + 1 }`.
#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LambdaFunctionParameter {
    pub name: IdentifierAST,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub ty: Option<Type>,
}

impl Expression {
    /// Returns the location of the expression.
    #[inline]
    #[must_use]
    pub const fn location(&self) -> Location {
        match self {
            Self::List { location, .. }
            | Self::As { location, .. }
            | Self::Binary { location, .. }
            | Self::StatementsBlock { location, .. }
            | Self::Literal(
                Literal::Integer { location, .. }
                | Literal::Float { location, .. }
                | Literal::Character { location, .. }
                | Literal::String { location, .. }
                | Literal::Boolean { location, .. },
            )
            | Self::Loop { location, .. }
            | Self::Identifier(IdentifierAST { location, .. })
            | Self::Parenthesized { location, .. }
            | Self::If { location, .. }
            | Self::FieldAccess { location, .. }
            | Self::Prefix { location, .. }
            | Self::Postfix { location, .. }
            | Self::While { location, .. }
            | Self::Call { location, .. }
            | Self::TypeArguments { location, .. }
            | Self::Tuple { location, .. }
            | Self::Struct { location, .. }
            | Self::Match { location, .. }
            | Self::Lambda { location, .. } => *location,
        }
    }
}

macro_rules! operator_type {
    {
        $(#[$($operator_type_doc:tt)*])*
        $operator_type_name:ident,

        $(#[$($raw_operator_type_doc:tt)*])*
        $raw_operator_type_name:ident,

        $(#[$($token_check_fn_doc:tt)*])*
        $token_check_fn_name:ident,

        $(
            $(#[$($doc:tt)*])*
            $str:literal => $name:ident
        ),*
    } => {
        $(#[$($operator_type_doc)*])*
        #[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
        #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
        pub struct $operator_type_name {
            #[cfg_attr(feature = "serde", serde(serialize_with = "use_display"))]
            #[cfg_attr(feature = "serde", serde(deserialize_with = "use_from_str"))]
            pub raw: $raw_operator_type_name,
            pub location: Location,
        }

        $(#[$($raw_operator_type_doc)*])*
        #[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
        pub enum $raw_operator_type_name {
            $(
                $(#[$($doc)*])*
                $name,
            )*
        }

        impl Display for $raw_operator_type_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", match self {
                    $(Self::$name => $str,)*
                })
            }
        }

        impl FromStr for $raw_operator_type_name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(stringify!($name) => Ok(Self::$name),)*
                    _ => Err("unexpected operator type".to_string()),
                }
            }
        }

        impl RawToken {
            $(#[$($raw_operator_type_doc)*])*
            #[inline]
            #[must_use]
            pub const fn $token_check_fn_name(self) -> bool {
                matches!(self, RawToken::Punctuator($(| Punctuator::$name)*))
            }
        }

        impl From<Punctuator> for $raw_operator_type_name {
            fn from(punctuator: Punctuator) -> Self {
                match punctuator {
                    $(Punctuator::$name => Self::$name,)*
                    _ => unreachable!(),
                }
            }
        }

        impl From<RawToken> for $raw_operator_type_name {
            fn from(token: RawToken) -> Self {
                if let RawToken::Punctuator(punctuator) = token {
                    punctuator.into()
                } else {
                    unreachable!()
                }
            }
        }

        impl From<$raw_operator_type_name> for Punctuator {
            fn from(value: $raw_operator_type_name) -> Self {
                match value {
                    $($raw_operator_type_name::$name => Punctuator::$name,)*
                }
            }
        }

        impl From<$raw_operator_type_name> for RawToken {
            fn from(value: $raw_operator_type_name) -> Self {
                RawToken::Punctuator(value.into()).into()
            }
        }

        impl From<$raw_operator_type_name> for String {
            fn from(value: $raw_operator_type_name) -> Self {
                RawToken::Punctuator(value.into()).into()
            }
        }
    };
}

operator_type! {
    /// A binary operator with a particular location.
    ///
    /// See [`RawBinaryOperator`] for more information.
    BinaryOperator,

    /// A binary operator, e.g. `+`, `**`, `/`.
    RawBinaryOperator,

    /// Returns `true` if the token is a binary operator.
    is_binary_operator,

    /// Plus Equal (`+=`).
    "+=" => PlusEq,

    /// Plus (`+`).
    "+" => Plus,

    /// Minus Equal (`-=`).
    "-=" => MinusEq,

    /// Minus (`-`).
    "-" => Minus,

    /// Asterisk Equal (`*=`).
    "*=" => AsteriskEq,

    /// Asterisk (`*`).
    "*" => Asterisk,

    /// Double Asterisk (`**`).
    "**" => DoubleAsterisk,

    /// Slash Equal (`/=`).
    "/=" => SlashEq,

    /// Slash (`/`).
    "/" => Slash,

    /// Bang Equal (`!=`).
    "!=" => BangEq,

    /// Right Shift (`>>`).
    ">>" => RightShift,

    /// Left Shift (`<<`).
    "<<" => LeftShift,

    /// Less Or Equal (`<=`).
    "<=" => LessEq,

    /// Less (`<`).
    "<" => Less,

    /// Greater Or Equal (`>=`).
    ">=" => GreaterEq,

    /// Greater (`>`).
    ">" => Greater,

    /// Double Equal (`==`).
    "==" => DoubleEq,

    /// Equal (`=`).
    "=" => Eq,

    /// Or (`|`).
    "|" => Or,

    /// Ampersand (`&`).
    "&" => Ampersand,

    /// Double Or (`||`).
    "||" => DoubleOr,

    /// Double Ampersand (`&&`).
    "&&" => DoubleAmpersand,

    /// Or Equal (`|=`).
    "|=" => OrEq,

    /// Ampersand Equal (`&=`).
    "&=" => AmpersandEq,

    /// Percent (`%`).
    "%" => Percent,

    /// Percent Equal (`%=`).
    "%=" => PercentEq
}

operator_type! {
    /// A prefix operator with a particular location.
    ///
    /// See [`RawPrefixOperator`] for more information.
    PrefixOperator,

    /// A prefix operator, e.g. `!`, `++`, `-`.
    RawPrefixOperator,

    /// Returns `true` if the token is a prefix operator.
    is_prefix_operator,

    /// Bang (`!`).
    "!" => Bang,

    /// Not (`~`).
    "~" => Tilde,

    /// Double Plus (`++`).
    "++" => DoublePlus,

    /// Double Minus (`--`).
    "--" => DoubleMinus,

    /// Plus (`+`).
    "+" => Plus,

    /// Minus (`-`).
    "-" => Minus
}

operator_type! {
    /// A postfix operator with a particular location.
    ///
    /// See [`RawPostfixOperator`] for more information.
    PostfixOperator,

    /// A postfix operator, e.g. `?`, `++`.
    RawPostfixOperator,

    /// Returns `true` if the token is a postfix operator.
    is_postfix_operator,

    /// Question Mark (`?`).
    "?" => QuestionMark,

    /// Double Plus (`++`).
    "++" => DoublePlus,

    /// Double Minus (`--`).
    "--" => DoubleMinus
}

/// A match expression item - `pattern` `=>` `expression`.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MatchExpressionItem {
    pub left: Pattern,
    pub right: Expression,
}

/// A field item in a struct expression (`identifier` and optionally `:` `expression`),
/// e.g. `name: "John"` and `age` in `Person { name: "John", age }`.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StructFieldExpression {
    pub name: IdentifierAST,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub value: Option<Expression>,
}

impl Expression {
    /// Returns `true` if this expression has a block in it (except function expressions).
    /// Used to determine if this expression has to have semicolon at the end.
    /// Function expression do have blocks in them, but they must have a semicolon at the end.
    #[inline]
    #[must_use]
    pub const fn with_block(&self) -> bool {
        matches!(
            self,
            Self::If { .. } | Self::While { .. } | Self::Match { .. }
        )
    }
}

/// A statement, e.g. `defer file.close()`, `return Some("hello");`, `break;`.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind"))]
pub enum Statement {
    /// Defer statement - `defer <expr>;`, e.g. `defer file.close()`.
    #[cfg_attr(feature = "serde", serde(rename = "defer_statement"))]
    Defer { call: Expression },

    /// Expression statement, e.g. `call();`.
    #[cfg_attr(feature = "serde", serde(rename = "expression_statement"))]
    Expression {
        expression: Expression,
        has_semicolon: bool,
    },

    /// Break statement - `break;`.
    #[cfg_attr(feature = "serde", serde(rename = "break_statement"))]
    Break { location: Location },

    /// Continue statement - `continue`;
    #[cfg_attr(feature = "serde", serde(rename = "continue_statement"))]
    Continue { location: Location },

    /// Return statement - `return <expr>;`, e.g. `return 42;`.
    #[cfg_attr(feature = "serde", serde(rename = "return_statement"))]
    Return { expression: Expression },

    /// Let statement - `let <pattern> = <expr>;`, e.g. `let x = 1`.
    #[cfg_attr(feature = "serde", serde(rename = "let_statement"))]
    Let {
        pattern: Pattern,
        value: Expression,

        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        #[cfg_attr(feature = "serde", serde(rename = "type"))]
        ty: Option<Type>,
    },
}

/// An interface module item.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Interface {
    pub visibility: Visibility,
    pub name: IdentifierAST,
    pub generic_parameters: Vec<GenericParameter>,
    pub where_predicates: Vec<WherePredicate>,
    pub methods: Vec<Function>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub inherits: Option<Vec<TypeConstructor>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub docstring: Option<String>,
}

/// An enum module item.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Enum {
    pub visibility: Visibility,
    pub name: IdentifierAST,
    pub generic_parameters: Vec<GenericParameter>,
    pub where_predicates: Vec<WherePredicate>,
    pub items: Vec<EnumItem>,
    pub methods: Vec<Function>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub implements: Option<Vec<TypeConstructor>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub docstring: Option<String>,
}

/// A struct module item.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Struct {
    pub visibility: Visibility,
    pub name: IdentifierAST,
    pub generic_parameters: Vec<GenericParameter>,
    pub where_predicates: Vec<WherePredicate>,
    pub fields: Vec<StructField>,
    pub methods: Vec<Function>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub implements: Option<Vec<TypeConstructor>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub docstring: Option<String>,
}

/// A tuple-like struct module item.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TupleLikeStruct {
    pub visibility: Visibility,
    pub name: IdentifierAST,
    pub generic_parameters: Vec<GenericParameter>,
    pub where_predicates: Vec<WherePredicate>,
    pub fields: Vec<TupleField>,
    pub methods: Vec<Function>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub implements: Option<Vec<TypeConstructor>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub docstring: Option<String>,
}

/// A module item.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind"))]
pub enum ModuleItem {
    /// An enum module item.
    #[cfg_attr(feature = "serde", serde(rename = "enum_item"))]
    Enum(Enum),

    /// A function module item.
    #[cfg_attr(feature = "serde", serde(rename = "function_item"))]
    Function(Function),

    /// An import module item.
    #[cfg_attr(feature = "serde", serde(rename = "import_item"))]
    Import {
        /// Location of the entire import item.
        location: Location,
        path: ImportPath,
    },

    /// An interface module item.
    #[cfg_attr(feature = "serde", serde(rename = "interface_item"))]
    Interface(Interface),

    /// A struct module item.
    #[cfg_attr(feature = "serde", serde(rename = "struct_item"))]
    Struct(Struct),

    /// A tuple-like struct module item.
    #[cfg_attr(feature = "serde", serde(rename = "tuple_like_struct_item"))]
    TupleLikeStruct(TupleLikeStruct),

    /// A type alias module item.
    #[cfg_attr(feature = "serde", serde(rename = "type_alias_item"))]
    TypeAlias(TypeAlias),
}

impl ModuleItem {
    /// Returns the location of the item.
    #[inline]
    #[must_use]
    pub const fn location(&self) -> Location {
        match self {
            Self::Enum(Enum {
                name: IdentifierAST { location, .. },
                ..
            })
            | Self::Function(Function {
                signature:
                    FunctionSignature {
                        name: IdentifierAST { location, .. },
                        ..
                    },
                ..
            })
            | Self::Import { location, .. }
            | Self::Struct(Struct {
                name: IdentifierAST { location, .. },
                ..
            })
            | Self::Interface(Interface {
                name: IdentifierAST { location, .. },
                ..
            })
            | Self::TupleLikeStruct(TupleLikeStruct {
                name: IdentifierAST { location, .. },
                ..
            })
            | Self::TypeAlias(TypeAlias {
                name: IdentifierAST { location, .. },
                ..
            }) => *location,
        }
    }

    /// Returns the id of the item name identifier.
    #[inline]
    #[must_use]
    pub const fn name_identifier_id(&self) -> Option<IdentifierID> {
        match self {
            Self::Enum(Enum {
                name: IdentifierAST { id, .. },
                ..
            })
            | Self::Function(Function {
                signature:
                    FunctionSignature {
                        name: IdentifierAST { id, .. },
                        ..
                    },
                ..
            })
            | Self::Struct(Struct {
                name: IdentifierAST { id, .. },
                ..
            })
            | Self::TupleLikeStruct(TupleLikeStruct {
                name: IdentifierAST { id, .. },
                ..
            })
            | Self::Interface(Interface {
                name: IdentifierAST { id, .. },
                ..
            })
            | Self::TypeAlias(TypeAlias {
                name: IdentifierAST { id, .. },
                ..
            }) => Some(*id),
            Self::Import { .. } => None,
        }
    }

    /// Returns the id of the item name identifier.
    ///
    /// # Panics
    ///
    /// If the item does not have a name.
    #[inline]
    #[must_use]
    pub fn name_identifier_id_or_panic(&self) -> IdentifierID {
        self.name_identifier_id().unwrap()
    }

    /// Returns the kind of the item.
    #[inline]
    #[must_use]
    pub const fn kind(&self) -> ModuleItemKind {
        match self {
            Self::Enum { .. } => ModuleItemKind::Enum,
            Self::Function(..) => ModuleItemKind::Function,
            Self::Import { .. } => ModuleItemKind::Import,
            Self::Interface { .. } => ModuleItemKind::Interface,
            Self::Struct { .. } => ModuleItemKind::Struct,
            Self::TupleLikeStruct { .. } => ModuleItemKind::TupleLikeStruct,
            Self::TypeAlias(..) => ModuleItemKind::TypeAlias,
        }
    }

    /// Returns the visibility of the item.
    #[inline]
    #[must_use]
    pub const fn visibility(&self) -> Option<Visibility> {
        match self {
            Self::Enum(Enum { visibility, .. })
            | Self::Struct(Struct { visibility, .. })
            | Self::TupleLikeStruct(TupleLikeStruct { visibility, .. })
            | Self::Interface(Interface { visibility, .. })
            | Self::TypeAlias(TypeAlias { visibility, .. })
            | Self::Function(Function {
                signature: FunctionSignature { visibility, .. },
                ..
            }) => Some(*visibility),
            Self::Import { .. } => None,
        }
    }

    /// Returns the visibility of the item.
    ///
    /// # Panics
    ///
    /// If the item does not have a visibility.
    #[inline]
    #[must_use]
    pub fn visibility_or_panic(&self) -> Visibility {
        self.visibility().unwrap()
    }
}

/// A kind of module item.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Display)]
pub enum ModuleItemKind {
    #[display(fmt = "enum")]
    Enum,

    #[display(fmt = "function")]
    Function,

    #[display(fmt = "import")]
    Import,

    #[display(fmt = "interface")]
    Interface,

    #[display(fmt = "struct")]
    Struct,

    #[display(fmt = "tuple-like struct")]
    TupleLikeStruct,

    #[display(fmt = "type alias")]
    TypeAlias,
}

/// An enum item, e.g. `None`, `Ok(T)`, `A { b: T }`.
#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind"))]
pub enum EnumItem {
    /// Just an identifier, e.g. `None` in `enum Option[T] { Some(T), None }`.
    #[cfg_attr(feature = "serde", serde(rename = "identifier_item"))]
    Just {
        name: IdentifierAST,

        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        docstring: Option<String>,
    },
    /// A tuple-like enum item, e.g. `None` in `enum Option<T> { Some(T), None }`.
    #[cfg_attr(feature = "serde", serde(rename = "tuple_like_item"))]
    TupleLike {
        name: IdentifierAST,
        fields: Vec<TupleField>,

        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        docstring: Option<String>,
    },
    /// A struct item, e.g. `A { b: T }` in `enum B { A { b: T } }`.
    #[cfg_attr(feature = "serde", serde(rename = "struct_item"))]
    Struct {
        name: IdentifierAST,
        fields: Vec<StructField>,

        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        docstring: Option<String>,
    },
}

/// A tuple field, e.g. `pub String` in `pub struct Wrapper(pub String);`.
#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TupleField {
    pub visibility: Visibility,

    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub ty: Type,
}

/// A struct field, e.g. `name: String`, `pub age: uint32`.
#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StructField {
    pub visibility: Visibility,
    pub name: IdentifierAST,

    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub ty: Type,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub docstring: Option<String>,
}

/// A function.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Function {
    pub signature: FunctionSignature,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub body: Option<Vec<Statement>>,
}

/// A function signature - information about function except a block.
#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FunctionSignature {
    pub visibility: Visibility,
    pub name: IdentifierAST,
    pub generic_parameters: Vec<GenericParameter>,
    pub parameters: Vec<FunctionParameter>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub return_type: Option<Type>,

    pub where_predicates: Vec<WherePredicate>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub docstring: Option<String>,
}

/// A function parameter, e.g. `self`, `self: Self`, `a: uint32`.
#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind"))]
pub enum FunctionParameter {
    /// A function parameter that is not `self`.
    #[cfg_attr(feature = "serde", serde(rename = "not_self"))]
    NotSelfParameter(NotSelfFunctionParameter),

    /// A self parameter.
    #[cfg_attr(feature = "serde", serde(rename = "self"))]
    SelfParameter(SelfFunctionParameter),
}

/// A self parameter, e.g. `self`, `self: Self`.
#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SelfFunctionParameter {
    pub self_location: Location,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub ty: Option<Type>,
}

/// A function parameter that is not `self`, e.g. `a: uint32`.
#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NotSelfFunctionParameter {
    pub name: IdentifierAST,

    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub ty: Type,
}

/// A Ry module.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Module {
    pub items: Vec<ModuleItem>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub docstring: Option<String>,
}

/// A visibility qualifier - `pub` or nothing (private visibility).
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind"))]
pub enum Visibility {
    #[default]
    #[cfg_attr(feature = "serde", serde(rename = "private"))]
    Private,

    #[cfg_attr(feature = "serde", serde(rename = "public"))]
    Public(#[cfg_attr(feature = "serde", serde(rename = "location"))] Location),
}

#[cfg(feature = "serde")]
fn use_display<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Display,
    S: Serializer,
{
    serializer.collect_str(value)
}

#[cfg(feature = "serde")]
fn use_from_str<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(serde::de::Error::custom)
}
