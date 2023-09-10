//! # HIR
//!
//! In Stellar, we have 2 abstract structures that represents relations between tokens - AST and HIR.
//! AST in this case is more abstract than HIR, because HIR is a more desugared version of AST,
//! e.g. `loop` expression doesn't exist for the HIR and is reduced into `while true {}` expression.
//!
//! For example, here is a code example, that uses `loop` expression:
//!
//! ```stellar
//! fun main() {
//!     loop {
//!         println("printing this forever!");
//!     }
//! }
//! ```
//!
//! After AST lowering (converting AST into HIR), the resulting HIR will looks like this:
//!
//! ```json
//! {
//!     "items": [
//!         {
//!             "kind": "function_module_item",
//!             "signature": {
//!                 "visibility": {
//!                     "kind": "private"
//!                 },
//!                 "name": {
//!                     "location": {
//!                         "filepath": "test.sr",
//!                         "start": 4,
//!                         "end": 8
//!                     },
//!                     "id": "main"
//!                 },
//!                 "generic_parameters": [],
//!                 "parameters": [],
//!                 "where_predicates": []
//!             },
//!             "body": [
//!                 {
//!                     "kind": "expression_statement",
//!                     "expression": {
//!                         "kind": "while_expression",
//!                         "location": {
//!                             "filepath": "test.sr",
//!                             "start": 17,
//!                             "end": 21
//!                         },
//!                         "condition": {
//!                             "kind": "literal_expression",
//!                             "literal_kind": "boolean",
//!                             "value": true,
//!                             "location": {
//!                                 "filepath": "test.sr",
//!                                 "start": 17,
//!                                 "end": 21
//!                             }
//!                         },
//!                         "statements_block": [
//!                             {
//!                                 "kind": "expression_statement",
//!                                 "expression": {
//!                                     "kind": "call_expression",
//!                                     "location": {
//!                                         "filepath": "test.sr",
//!                                         "start": 32,
//!                                         "end": 65
//!                                     },
//!                                     "callee": {
//!                                         "kind": "identifier_expression",
//!                                         "location": {
//!                                             "filepath": "test.sr",
//!                                             "start": 32,
//!                                             "end": 39
//!                                         },
//!                                         "id": "println"
//!                                     },
//!                                     "arguments": [
//!                                         {
//!                                             "kind": "literal_expression",
//!                                             "literal_kind": "string",
//!                                             "value": "printing this forever!",
//!                                             "location": {
//!                                                 "filepath": "test.sr",
//!                                                 "start": 40,
//!                                                 "end": 64
//!                                             }
//!                                         }
//!                                     ]
//!                                 },
//!                                 "has_semicolon": true
//!                             }
//!                         ]
//!                     },
//!                     "has_semicolon": false
//!                 }
//!             ]
//!         }
//!     ]
//! }
//! ```
//!
//! In this case, we can see that `loop` was converted into `while true`:
//!
//! ```json
//!                         "kind": "while_expression",
//!                         "location": {
//!                             "filepath": "test.sr",
//!                             "start": 17,
//!                             "end": 21
//!                         },
//!                         "condition": {
//!                             "kind": "literal_expression",
//!                             "literal_kind": "boolean",
//!                             "value": true,
//!                             "location": {
//!                                 "filepath": "test.sr",
//!                                 "start": 17,
//!                                 "end": 21
//!                             }
//!                         },
//! ```

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/quantumatic/stellar/main/additional/icon/stellar.png",
    html_favicon_url = "https://raw.githubusercontent.com/quantumatic/stellar/main/additional/icon/stellar.png"
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
    unused_crate_dependencies,
    unused_import_braces,
    unused_lifetimes,
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
    clippy::unnested_or_patterns,
    clippy::inline_always
)]

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
pub use stellar_ast::{IdentifierAST, ImportPath, Literal, Path, Visibility};
use stellar_ast::{ModuleItemKind, NegativeNumericLiteral};
use stellar_filesystem::location::Location;
use stellar_interner::{IdentifierId, PathId};

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

    /// A negative numeric literal pattern, e.g. `-3`.
    #[cfg_attr(feature = "serde", serde(rename = "negative_numeric_literal_pattern"))]
    NegativeNumericLiteral(NegativeNumericLiteral),

    /// An identifier pattern, e.g. `f`, `list @ [3, ..]`.
    #[cfg_attr(feature = "serde", serde(rename = "identifier_pattern"))]
    Identifier {
        location: Location,
        identifier: IdentifierAST,

        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pattern: Option<Box<Self>>,
    },

    /// A wildcard pattern, e.g. `_`.
    #[cfg_attr(feature = "serde", serde(rename = "wildcard_pattern"))]
    Wildcard { location: Location },

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
    #[inline(always)]
    #[must_use]
    pub const fn location(&self) -> Location {
        match self {
            Self::Identifier { location, .. }
            | Self::NegativeNumericLiteral(
                NegativeNumericLiteral::Float { location, .. }
                | NegativeNumericLiteral::Integer { location, .. },
            )
            | Self::List { location, .. }
            | Self::Or { location, .. }
            | Self::Rest { location, .. }
            | Self::Struct { location, .. }
            | Self::Tuple { location, .. }
            | Self::TupleLike { location, .. }
            | Self::Path {
                path: Path { location, .. },
                ..
            }
            | Self::Wildcard { location } => *location,
            Self::Literal(literal) => literal.location(),
        }
    }
}

/// A pattern used to match a struct field, e.g. `citizenship: "USA"`, `name` and `..` in
/// `Person { citizenship: "USA", name, .. }`
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind"))]
pub enum StructFieldPattern {
    /// A pattern used to match a struct field, which is not rest pattern (`..`),
    /// e.g. `citizen: "USA"` and `name` in `Person { citizen: "USA", name, .. }`.
    #[cfg_attr(feature = "serde", serde(rename = "not_rest_pattern"))]
    NotRest {
        location: Location,
        field_name: IdentifierAST,

        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        value_pattern: Option<Pattern>,
    },

    #[cfg_attr(feature = "serde", serde(rename = "rest_pattern"))]
    /// A rest pattern, e.g. `..`.
    Rest { location: Location },
}

/// A type, e.g. `int32`, `(char): bool`, `(char, char)`.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind"))]
pub enum Type {
    /// A type constructor, e.g. `char`, `List[int32]`.
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
        return_type: Option<Box<Self>>,
    },

    /// An underscore type, e.g. `_`.
    #[cfg_attr(feature = "serde", serde(rename = "underscore_type"))]
    Underscore { location: Location },

    /// An interface object type, e.g. `dyn Iterator[Item = uint32]`, `dyn Debug + Clone`.
    #[cfg_attr(feature = "serde", serde(rename = "interface_object_type"))]
    InterfaceObject {
        location: Location,
        bounds: Vec<TypeConstructor>,
    },
}

impl Type {
    /// Returns the location of the type.
    #[inline(always)]
    #[must_use]
    pub const fn location(&self) -> Location {
        match self {
            Self::Function { location, .. }
            | Self::Constructor(TypeConstructor { location, .. })
            | Self::InterfaceObject { location, .. }
            | Self::Tuple { location, .. }
            | Self::Underscore { location } => *location,
        }
    }
}

/// A generic parameter, e.g. `T` in `fun into[T](a: T);`.
#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GenericParameter {
    pub name: IdentifierAST,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub bounds: Option<Vec<TypeConstructor>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub default_value: Option<Type>,
}

/// A type alias, e.g. `type MyResult = Result[String, MyError]`.
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

/// A where clause item, e.g. `T: ToString`.
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

    /// Binary expression, e.g. `1 + 2`.
    #[cfg_attr(feature = "serde", serde(rename = "binary_expression"))]
    Binary {
        location: Location,
        left: Box<Self>,
        operator: stellar_ast::BinaryOperator,
        right: Box<Self>,
    },

    /// Block expression, e.g. `{ let b = 1; b }`.
    #[cfg_attr(feature = "serde", serde(rename = "block_expression"))]
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

    /// Underscore expression, e.g. `_`.
    #[cfg_attr(feature = "serde", serde(rename = "underscore_expression"))]
    Underscore { location: Location },

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
        operator: stellar_ast::PrefixOperator,
    },

    /// Postfix expression, e.g. `safe_div(1, 0)?`, `a++`.
    #[cfg_attr(feature = "serde", serde(rename = "postfix_expression"))]
    Postfix {
        location: Location,
        inner: Box<Self>,
        operator: stellar_ast::PostfixOperator,
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

    /// Type expression, e.g. `A[int32]`.
    TypeArguments {
        location: Location,
        left: Box<Self>,
        type_arguments: Vec<Type>,
    },

    /// Tuple expression, e.g. `(a, 32, \"hello\")`.
    Tuple {
        location: Location,
        elements: Vec<Self>,
    },

    /// Struct expression, e.g. `Person { name: \"John\", age: 25 }`.
    Struct {
        location: Location,
        left: Box<Self>,
        fields: Vec<StructExpressionItem>,
    },

    /// Match expression (`match fs.read_file(...) { ... }`).
    Match {
        location: Location,
        expression: Box<Self>,
        block: Vec<MatchExpressionItem>,
    },

    /// Lambda expression (`|x| { x + 1 }`).
    Lambda {
        location: Location,
        parameters: Vec<LambdaFunctionParameter>,
        return_type: Option<Type>,
        value: Box<Self>,
    },
}

/// A lambda function parameter, e.g. `x` in `|x| { x + 1 }`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LambdaFunctionParameter {
    pub name: IdentifierAST,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub ty: Option<Type>,
}

impl Expression {
    /// Returns the location of the expression.
    #[inline(always)]
    #[must_use]
    pub const fn location(&self) -> Location {
        match self {
            Self::List { location, .. }
            | Self::As { location, .. }
            | Self::Binary { location, .. }
            | Self::StatementsBlock { location, .. }
            | Self::Identifier(IdentifierAST { location, .. })
            | Self::If { location, .. }
            | Self::FieldAccess { location, .. }
            | Self::Prefix { location, .. }
            | Self::Postfix { location, .. }
            | Self::While { location, .. }
            | Self::Call { location, .. }
            | Self::Tuple { location, .. }
            | Self::Struct { location, .. }
            | Self::Match { location, .. }
            | Self::Lambda { location, .. }
            | Self::TypeArguments { location, .. }
            | Self::Underscore { location } => *location,
            Self::Literal(literal) => literal.location(),
        }
    }
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
pub struct StructExpressionItem {
    pub name: IdentifierAST,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub value: Option<Expression>,
}

impl Expression {
    /// Returns `true` if this expression has a block in it (except function expressions).
    /// Used to determine if this expression has to have semicolon at the end.
    /// Function expression do have blocks in them, but they must have a semicolon at the end.
    #[inline(always)]
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

/// A function.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Function {
    pub signature: FunctionSignature,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub body: Option<Vec<Statement>>,
}

/// A function signature - information about function except a block.
#[derive(Debug, PartialEq, Clone)]
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
    /// Enum item.
    #[cfg_attr(feature = "serde", serde(rename = "enum_module_item"))]
    Enum(Enum),

    /// Function item.
    #[cfg_attr(feature = "serde", serde(rename = "function_module_item"))]
    Function(Function),

    /// Import item.
    #[cfg_attr(feature = "serde", serde(rename = "import_module_item"))]
    Import {
        /// Location of the entire import item.
        location: Location,
        path: ImportPath,
    },

    /// Interface item.
    #[cfg_attr(feature = "serde", serde(rename = "interface_module_item"))]
    Interface(Interface),

    /// Struct item.
    #[cfg_attr(feature = "serde", serde(rename = "struct_module_item"))]
    Struct(Struct),

    /// Tuple-like struct item.
    #[cfg_attr(feature = "serde", serde(rename = "tuple_like_struct_module_item"))]
    TupleLikeStruct(TupleLikeStruct),

    /// Type alias item.
    #[cfg_attr(feature = "serde", serde(rename = "type_alias_module_item"))]
    TypeAlias(TypeAlias),
}

impl ModuleItem {
    /// Returns the location of the item.
    #[inline(always)]
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

    /// Returns the location of the item.
    #[inline(always)]
    #[must_use]
    pub const fn name(&self) -> Option<IdentifierId> {
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
    #[inline(always)]
    #[must_use]
    pub fn name_or_panic(&self) -> IdentifierId {
        self.name().unwrap()
    }

    /// Returns the kind of the item.
    #[inline(always)]
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
    #[inline(always)]
    #[must_use]
    pub const fn visibility(&self) -> Visibility {
        match self {
            Self::Enum(Enum { visibility, .. })
            | Self::Struct(Struct { visibility, .. })
            | Self::TupleLikeStruct(TupleLikeStruct { visibility, .. })
            | Self::Interface(Interface { visibility, .. })
            | Self::TypeAlias(TypeAlias { visibility, .. })
            | Self::Function(Function {
                signature: FunctionSignature { visibility, .. },
                ..
            }) => *visibility,
            Self::Import { .. } => Visibility::Private,
        }
    }

    /// Returns the type alias variant of the time.
    #[inline(always)]
    #[must_use]
    pub const fn type_alias(&self) -> Option<&TypeAlias> {
        match self {
            Self::TypeAlias(alias) => Some(alias),
            _ => None,
        }
    }

    /// Returns the type alias variant of the time.
    ///
    /// # Panics
    /// If the item is not a type alias.
    #[inline(always)]
    #[must_use]
    pub fn type_alias_or_panic(&self) -> &TypeAlias {
        self.type_alias()
            .unwrap_or_else(|| panic!("expected type alias, got {}", self.kind()))
    }
}

/// An enum item, e.g. `None`, `Ok(T)`, `A { b: T }`.
#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind"))]
pub enum EnumItem {
    /// Just an identifier, e.g. `None` in `enum Option[T] { Some(T), None }`.
    #[cfg_attr(feature = "serde", serde(rename = "identifier"))]
    Just {
        name: IdentifierAST,

        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        docstring: Option<String>,
    },
    /// A tuple-like enum item, e.g. `None` in `enum Option<T> { Some(T), None }`.
    #[cfg_attr(feature = "serde", serde(rename = "tuple_like"))]
    TupleLike {
        name: IdentifierAST,
        fields: Vec<TupleField>,

        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        docstring: Option<String>,
    },
    /// A struct item, e.g. `A { b: T }` in `enum B { A { b: T } }`.
    #[cfg_attr(feature = "serde", serde(rename = "struct"))]
    Struct {
        name: IdentifierAST,
        fields: Vec<StructField>,

        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        docstring: Option<String>,
    },
}

impl EnumItem {
    #[inline(always)]
    #[must_use]
    pub const fn name(&self) -> IdentifierAST {
        match self {
            Self::Just { name, .. } | Self::TupleLike { name, .. } | Self::Struct { name, .. } => {
                *name
            }
        }
    }

    #[inline(always)]
    #[must_use]
    pub const fn name_id(&self) -> IdentifierId {
        self.name().id
    }
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

/// A function parameter, e.g. `self`, `self: Self`, `a: uint32`.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NotSelfFunctionParameter {
    pub pattern: Pattern,

    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub ty: Type,
}

/// A Stellar module.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Module {
    pub filepath: PathId,

    pub items: Vec<ModuleItem>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub docstring: Option<String>,
}
