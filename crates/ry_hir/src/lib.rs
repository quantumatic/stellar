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
//! For more details see the module items and start with [`Module`] node.
//!
//! # Serialization
//!
//! AST can be serialized into a string using [`serialize_ast()`]. This is used in the
//! language CLI `parse` command, when serialized AST is written into a txt file.
//!
//! See [`Serializer`] for more details.
//!
//! [`Serializer`]: crate::serialize::Serializer
//! [`serialize_ast()`]: crate::serialize::serialize_ast
//! [`Token`]: crate::token::Token
//! [`ry_parser`]: ../ry_parser/index.html

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png",
    html_favicon_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png"
)]
#![warn(clippy::dbg_macro)]
#![deny(
    // rustc lint groups https://doc.rust-lang.org/rustc/lints/groups.html
    warnings,
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
    clippy::unnested_or_patterns
)]

pub mod ty;

use std::fmt::Display;

use ry_ast::{IdentifierAST, ImportPath, Path, TypeBounds, Visibility};
use ry_filesystem::location::Location;
use ry_interner::Symbol;
use ty::{Type, Typed};

#[derive(Debug, PartialEq, Clone)]
pub struct Literal {
    pub literal: ry_ast::Literal,
    pub ty: Type,
}

impl Literal {
    #[inline]
    #[must_use]
    pub const fn location(&self) -> Location {
        self.literal.location()
    }
}

impl Typed for Literal {
    #[inline]
    fn ty(&self) -> Type {
        self.ty.clone()
    }
}

/// A pattern, e.g. `Some(x)`, `None`, `a @ [3, ..]`, `[1, .., 3]`, `(1, \"hello\")`, `3.2`.
#[derive(Debug, PartialEq, Clone)]
pub enum Pattern {
    /// A literal pattern, e.g. `3.14`, `'a'`, `true`.
    Literal(Literal),

    /// An identifier pattern, e.g. `f`, `list @ [3, ..]`.
    Identifier {
        location: Location,
        identifier: IdentifierAST,
        pattern: Option<Box<Self>>,
        ty: Type,
    },

    /// A struct pattern, e.g. `Person { name, age, .. }`.
    Struct {
        location: Location,
        path: Path,
        fields: Vec<StructFieldPattern>,
        ty: Type,
    },

    /// A tuple-like pattern - used to match a tuple-like structs and enum tuple-like items,
    /// e.g. `Some(x)`, `A()`.
    TupleLike {
        location: Location,
        path: Path,
        inner_patterns: Vec<Self>,
        ty: Type,
    },

    /// A tuple pattern, e.g. `(a, "hello", ..)`.
    Tuple {
        location: Location,
        elements: Vec<Self>,
        ty: Type,
    },

    /// A path pattern.
    Path { path: Path, ty: Type },

    /// A list pattern, e.g. `[1, .., 10]`.
    List {
        location: Location,
        inner_patterns: Vec<Self>,
        ty: Type,
    },

    /// An or pattern, e.g. `Some(..) | None`.
    Or {
        location: Location,
        left: Box<Self>,
        right: Box<Self>,
        ty: Type,
    },

    /// A rest pattern - `..`.
    Rest { location: Location, ty: Type },
}

impl Pattern {
    /// Returns the location of the pattern.
    #[inline]
    #[must_use]
    pub const fn location(&self) -> Location {
        match self {
            Self::Identifier { location, .. }
            | Self::List { location, .. }
            | Self::Or { location, .. }
            | Self::Rest { location, .. }
            | Self::Struct { location, .. }
            | Self::Tuple { location, .. }
            | Self::TupleLike { location, .. }
            | Self::Path {
                path: Path { location, .. },
                ..
            } => *location,
            Self::Literal(literal) => literal.location(),
        }
    }
}

/// A pattern used to match a struct field, e.g. `citizenship: "USA"`, `name` and `..` in
/// `Person { citizenship: "USA", name, .. }`
#[derive(Debug, PartialEq, Clone)]
pub enum StructFieldPattern {
    /// A pattern used to match a struct field, which is not rest pattern (`..`),
    /// e.g. `citizen: "USA"` and `name` in `Person { citizen: "USA", name, .. }`.
    NotRest {
        location: Location,
        field_name: IdentifierAST,
        value_pattern: Option<Pattern>,
        ty: Type,
    },
    /// A rest pattern, e.g. `..`.
    Rest { location: Location, ty: Type },
}

/// A type, e.g. `int32`, `[S, dyn Iterator[Item = uint32]]`, `(char, char)`.
#[derive(Debug, PartialEq, Clone)]
pub enum TypeExpression {
    /// A type path, e.g. `Iterator[Item = uint32].Item`, `R.Output`, `char`.
    Path(ry_ast::TypePath),

    /// A tuple type, e.g. `(int32, String, char)`.
    Tuple {
        location: Location,
        element_types: Vec<Self>,
    },

    /// A function type (return type is required for consistency), e.g. `(char): bool`.
    Function {
        location: Location,
        parameter_types: Vec<Self>,
        return_type: Box<Self>,
    },

    /// A trait object type, e.g. `dyn Iterator[Item = uint32]`, `dyn Debug + Clone`.
    TraitObject {
        location: Location,
        bounds: ry_ast::TypeBounds,
    },

    /// A type with a qualified path, e.g. `[A as Iterator].Item`.
    WithQualifiedPath {
        location: Location,
        left: Box<Self>,
        right: ry_ast::TypePath,
        segments: Vec<ry_ast::TypePathSegment>,
    },
}

impl TypeExpression {
    /// Returns the location of the type.
    #[inline]
    #[must_use]
    pub const fn location(&self) -> Location {
        match self {
            Self::Function { location, .. }
            | Self::Path(ry_ast::TypePath { location, .. })
            | Self::TraitObject { location, .. }
            | Self::Tuple { location, .. }
            | Self::WithQualifiedPath { location, .. } => *location,
        }
    }
}

/// A type parameter, e.g. `T` in `fun into[T](a: T);`.
#[derive(Debug, PartialEq, Clone)]
pub struct TypeParameter {
    pub name: IdentifierAST,
    pub bounds: Option<ry_ast::TypeBounds>,
    pub default_value_type_expression: Option<TypeExpression>,
    pub ty: Type,
}

/// A type alias, e.g. `type MyResult = Result[String, MyError]`.
#[derive(Debug, PartialEq, Clone)]
pub struct TypeAlias {
    pub visibility: Visibility,
    pub name: IdentifierAST,
    pub type_parameters: Vec<TypeParameter>,
    pub bounds: Option<ry_ast::TypeBounds>,
    pub value_type_expression: Option<TypeExpression>,
    pub value: Type,
    pub docstring: Option<String>,
}

/// A where clause item, e.g. `T: Into<String>` and `[T as Iterator].Item = char` in
/// `where T: Into<String>, [T as Iterator].Item = char`.
#[derive(Debug, PartialEq, Clone)]
pub enum WherePredicate {
    Eq {
        left_type_expression: TypeExpression,
        left_type: Type,
        right_type_expression: TypeExpression,
        right_type: Type,
    },
    Satisfies {
        type_expression: TypeExpression,
        ty: Type,
        bounds: ry_ast::TypeBounds,
    },
}

/// An expression.
#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    /// List expression, e.g. `[1, 2, 3]`.
    List {
        location: Location,
        elements: Vec<Self>,
        ty: Type,
    },

    /// As expression, e.g. `a as float32`.
    As {
        location: Location,
        left: Box<Self>,
        right: TypeExpression,
        ty: Type,
    },

    /// Binary expression, e.g. `1 + 2`.
    Binary {
        location: Location,
        left: Box<Self>,
        operator: ry_ast::BinaryOperator,
        right: Box<Self>,
        ty: Type,
    },

    /// Block expression, e.g. `{ let b = 1; b }`.
    StatementsBlock {
        location: Location,
        block: Vec<Statement>,
        ty: Type,
    },

    /// Literal expression, e.g. `true`, `\"hello\"`, `1.2`.
    Literal(Literal),

    /// Identifier expression, e.g. `foo`.
    Identifier(IdentifierAST),

    /// Parenthesized expression, e.g. `(1 + 2)`.
    Parenthesized {
        location: Location,
        inner: Box<Self>,
        ty: Type,
    },

    /// If expression, e.g. `if x { ... } else { ... }`.
    If {
        location: Location,
        if_blocks: Vec<(Self, Vec<Statement>)>,
        r#else: Option<Vec<Statement>>,
        ty: Type,
    },

    /// Field access expression, e.g. `x.y`.
    FieldAccess {
        location: Location,
        left: Box<Self>,
        right: IdentifierAST,
        ty: Type,
    },

    /// Prefix expression, e.g. `!false`, `++a`.
    Prefix {
        location: Location,
        inner: Box<Self>,
        operator: ry_ast::PrefixOperator,
        ty: Type,
    },

    /// Postfix expression, e.g. `safe_div(1, 0)?`, `a++`.
    Postfix {
        location: Location,
        inner: Box<Self>,
        operator: ry_ast::PostfixOperator,
        ty: Type,
    },

    /// While expression, e.g. `while x != 0 {}`.
    While {
        location: Location,
        condition: Box<Self>,
        statements_block: Vec<Statement>,
        ty: Type,
    },

    /// Call expression, e.g. `s.to_string()`.
    Call {
        location: Location,
        callee: Box<Self>,
        arguments: Vec<Self>,
        ty: Type,
    },

    TypeArguments {
        location: Location,
        left: Box<Self>,
        type_arguments: Vec<TypeArgument>,
        ty: Type,
    },

    TypeFieldAccess {
        location: Location,
        left: Type,
        right: IdentifierAST,
        ty: Type,
    },

    /// Tuple expression, e.g. `(a, 32, \"hello\")`.
    Tuple {
        location: Location,
        elements: Vec<Self>,
        ty: Type,
    },

    /// Struct expression, e.g. `Person { name: \"John\", age: 25 }`.
    Struct {
        location: Location,
        left: Box<Self>,
        fields: Vec<StructExpressionItem>,
        ty: Type,
    },

    /// Match expression (`match fs.read_file(...) { ... }`).
    Match {
        location: Location,
        expression: Box<Self>,
        block: Vec<MatchExpressionItem>,
        ty: Type,
    },

    /// Lambda expression (`|x| { x + 1 }`).
    Lambda {
        location: Location,
        parameters: Vec<LambdaFunctionParameter>,
        return_type_expression: Option<TypeExpression>,
        block: Vec<Statement>,
        ty: Type,
    },
}

/// A lambda function parameter, e.g. `x` in `|x| { x + 1 }`.
#[derive(Debug, Clone, PartialEq)]
pub struct LambdaFunctionParameter {
    pub name: IdentifierAST,
    pub type_expression: Option<TypeExpression>,
    pub ty: Type,
}

/// A type argument, e.g. `Item = uint32` in `Iterator[Item = uint32]`, `usize` in `sizeof[usize]()`.
#[derive(Debug, PartialEq, Clone)]
pub enum TypeArgument {
    /// Just a type, e.g. `usize` in `sizeof[usize]()`.
    Type {
        type_expression: TypeExpression,
        ty: Type,
    },
    /// Type with a name, e.g. `Item = uint32` in `Iterator[Item = uint32]`.
    AssociatedType {
        name: IdentifierAST,
        value_type_expression: TypeExpression,
        value: Type,
    },
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
            | Self::Identifier(IdentifierAST { location, .. })
            | Self::Parenthesized { location, .. }
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
            | Self::TypeFieldAccess { location, .. } => *location,
            Self::Literal(literal) => literal.location(),
        }
    }
}

/// A match expression item - `pattern` `=>` `expression`.
#[derive(Debug, PartialEq, Clone)]
pub struct MatchExpressionItem {
    pub left: Pattern,
    pub right: Expression,
}

/// A field item in a struct expression (`identifier` and optionally `:` `expression`),
/// e.g. `name: "John"` and `age` in `Person { name: "John", age }`.
#[derive(Debug, PartialEq, Clone)]
pub struct StructExpressionItem {
    pub name: IdentifierAST,
    pub value: Option<Expression>,
    pub ty: Type,
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
pub enum Statement {
    /// Defer statement - `defer <expr>;`, e.g. `defer file.close()`.
    Defer { call: Expression },

    /// Expression statement, e.g. `call();`.
    Expression {
        expression: Expression,
        has_semicolon: bool,
    },

    /// Break statement - `break;`.
    Break { location: Location },

    /// Continue statement - `continue`;
    Continue { location: Location },

    /// Return statement - `return <expr>;`, e.g. `return 42;`.
    Return { expression: Expression },

    /// Let statement - `let <pattern> = <expr>;`, e.g. `let x = 1`.
    Let {
        pattern: Pattern,
        value: Expression,
        type_expression: Option<TypeExpression>,
        ty: Type,
    },
}

/// A block of statements - `{ <stmt>* }`.
pub type StatementsBlock = Vec<Statement>;

/// A type implementation.
#[derive(Debug, Clone, PartialEq)]
pub struct Impl {
    /// Location of the `impl` keyword.
    pub location: Location,
    pub type_parameters: Vec<TypeParameter>,
    pub ty: TypeExpression,
    pub r#trait: Option<ry_ast::TypePath>,
    pub where_predicates: Vec<WherePredicate>,
    pub items: Vec<TraitItem>,
    pub docstring: Option<String>,
}

/// A function.
#[derive(Debug, PartialEq, Clone)]
pub struct Function {
    pub signature: FunctionSignature,
    pub body: Option<StatementsBlock>,
}

/// A function signature - information about function except a block.
#[derive(Debug, PartialEq, Clone)]
pub struct FunctionSignature {
    pub visibility: Visibility,
    pub name: IdentifierAST,
    pub type_parameters: Vec<TypeParameter>,
    pub parameters: Vec<FunctionParameter>,
    pub return_type_expression: Option<TypeExpression>,
    pub return_type: Type,
    pub where_predicates: Vec<WherePredicate>,
    pub docstring: Option<String>,
}

/// A module item.
#[derive(Debug, PartialEq, Clone)]
pub enum ModuleItem {
    /// Enum item.
    Enum {
        visibility: Visibility,
        name: IdentifierAST,
        type_parameters: Vec<TypeParameter>,
        where_predicates: Vec<WherePredicate>,
        items: Vec<EnumItem>,
        docstring: Option<String>,
    },

    /// Function item.
    ///
    Function(Function),

    /// Import item.
    ///
    Import {
        /// Location of the entire import item.
        location: Location,
        path: ImportPath,
    },

    /// Trait item.
    Trait {
        visibility: Visibility,
        name: IdentifierAST,
        type_parameters: Vec<TypeParameter>,
        where_predicates: Vec<WherePredicate>,
        items: Vec<TraitItem>,
        docstring: Option<String>,
    },

    /// Impl item.
    Impl(Impl),

    /// Struct item.
    Struct {
        visibility: Visibility,
        name: IdentifierAST,
        type_parameters: Vec<TypeParameter>,
        where_predicates: Vec<WherePredicate>,
        fields: Vec<StructField>,
        docstring: Option<String>,
    },

    /// Tuple-like struct item.
    TupleLikeStruct {
        visibility: Visibility,
        name: IdentifierAST,
        type_parameters: Vec<TypeParameter>,
        where_predicates: Vec<WherePredicate>,
        fields: Vec<TupleField>,
        docstring: Option<String>,
    },

    /// Type alias item.
    TypeAlias(TypeAlias),
}

impl ModuleItem {
    /// Returns the location of the item.
    #[inline]
    #[must_use]
    pub const fn location(&self) -> Location {
        match self {
            Self::Enum {
                name: IdentifierAST { location, .. },
                ..
            }
            | Self::Function(Function {
                signature:
                    FunctionSignature {
                        name: IdentifierAST { location, .. },
                        ..
                    },
                ..
            })
            | Self::Impl(Impl { location, .. })
            | Self::Import { location, .. }
            | Self::Struct {
                name: IdentifierAST { location, .. },
                ..
            }
            | Self::Trait {
                name: IdentifierAST { location, .. },
                ..
            }
            | Self::TupleLikeStruct {
                name: IdentifierAST { location, .. },
                ..
            }
            | Self::TypeAlias(TypeAlias {
                name: IdentifierAST { location, .. },
                ..
            }) => *location,
        }
    }

    /// Returns the name of the item.
    #[inline]
    #[must_use]
    pub const fn name(&self) -> Option<Symbol> {
        match self {
            Self::Enum {
                name: IdentifierAST { symbol, .. },
                ..
            }
            | Self::Function(Function {
                signature:
                    FunctionSignature {
                        name: IdentifierAST { symbol, .. },
                        ..
                    },
                ..
            })
            | Self::Struct {
                name: IdentifierAST { symbol, .. },
                ..
            }
            | Self::TupleLikeStruct {
                name: IdentifierAST { symbol, .. },
                ..
            }
            | Self::Trait {
                name: IdentifierAST { symbol, .. },
                ..
            }
            | Self::TypeAlias(TypeAlias {
                name: IdentifierAST { symbol, .. },
                ..
            }) => Some(*symbol),
            Self::Import { .. } | Self::Impl(..) => None,
        }
    }

    /// Returns the name of the item.
    ///
    /// # Panics
    ///
    /// If the item does not have a name.
    #[inline]
    #[must_use]
    pub fn name_or_panic(&self) -> Symbol {
        self.name().unwrap()
    }

    /// Returns the kind of the item.
    #[inline]
    #[must_use]
    pub const fn kind(&self) -> ModuleItemKind {
        match self {
            Self::Enum { .. } => ModuleItemKind::Enum,
            Self::Function(..) => ModuleItemKind::Function,
            Self::Import { .. } => ModuleItemKind::Import,
            Self::Trait { .. } => ModuleItemKind::Trait,
            Self::Impl(..) => ModuleItemKind::Impl,
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
            Self::Enum { visibility, .. }
            | Self::Struct { visibility, .. }
            | Self::TupleLikeStruct { visibility, .. }
            | Self::Trait { visibility, .. }
            | Self::TypeAlias(TypeAlias { visibility, .. })
            | Self::Function(Function {
                signature: FunctionSignature { visibility, .. },
                ..
            }) => Some(*visibility),
            _ => None,
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
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ModuleItemKind {
    Enum,
    Function,
    Import,
    Trait,
    Impl,
    Struct,
    TupleLikeStruct,
    TypeAlias,
}

impl AsRef<str> for ModuleItemKind {
    fn as_ref(&self) -> &str {
        match self {
            Self::Enum => "enum",
            Self::Function => "function",
            Self::Import => "import",
            Self::Trait => "trait",
            Self::Impl => "type implementation",
            Self::Struct => "struct",
            Self::TupleLikeStruct => "tuple-like struct",
            Self::TypeAlias => "type alias",
        }
    }
}

impl Display for ModuleItemKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}

/// An enum item, e.g. `None`, `Ok(T)`, `A { b: T }`.
#[derive(Debug, PartialEq, Clone)]
pub enum EnumItem {
    /// Just an identifier, e.g. `None` in `enum Option[T] { Some(T), None }`.
    Just {
        name: IdentifierAST,
        docstring: Option<String>,
    },
    /// A tuple-like enum item, e.g. `None` in `enum Option<T> { Some(T), None }`.
    TupleLike {
        name: IdentifierAST,
        fields: Vec<TupleField>,
        docstring: Option<String>,
    },
    /// A struct item, e.g. `A { b: T }` in `enum B { A { b: T } }`.
    Struct {
        name: IdentifierAST,
        fields: Vec<StructField>,
        docstring: Option<String>,
    },
}

/// A tuple field, e.g. `pub String` in `pub struct Wrapper(pub String);`.
#[derive(Debug, PartialEq, Clone)]
pub struct TupleField {
    pub visibility: Visibility,
    pub type_expression: TypeExpression,
    pub ty: Type,
}

/// A struct field, e.g. `name: String`, `pub age: uint32`.
#[derive(Debug, PartialEq, Clone)]
pub struct StructField {
    pub visibility: Visibility,
    pub name: IdentifierAST,
    pub type_expression: TypeExpression,
    pub ty: Type,
    pub docstring: Option<String>,
}

/// A trait item - type alias or a function.
#[derive(Debug, PartialEq, Clone)]
pub enum TraitItem {
    /// Type alias item.
    TypeAlias(TypeAlias),

    /// Function item.
    AssociatedFunction(Function),
}

/// A function parameter, e.g. `self`, `self: Self`, `a: uint32`.
#[derive(Debug, PartialEq, Clone)]
pub enum FunctionParameter {
    /// A function parameter that is not `self`.
    NotSelfParameter(NotSelfFunctionParameter),

    /// A self parameter.
    SelfParameter(SelfFunctionParameter),
}

/// A self parameter, e.g. `self`, `self: Self`.
#[derive(Debug, PartialEq, Clone)]
pub struct SelfFunctionParameter {
    pub self_location: Location,
    pub type_expression: Option<TypeExpression>,
    pub ty: Type,
}

/// A function parameter that is not `self`, e.g. `a: uint32`.
#[derive(Debug, PartialEq, Clone)]
pub struct NotSelfFunctionParameter {
    pub name: IdentifierAST,
    pub ty: FunctionParameterType,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FunctionParameterType {
    Impl(TypeBounds),
    Type {
        type_expression: TypeExpression,
        ty: Type,
    },
}

/// A Ry module.
#[derive(Debug, PartialEq, Clone)]
pub struct Module {
    pub items: Vec<ModuleItem>,
    pub docstring: Option<String>,
}
