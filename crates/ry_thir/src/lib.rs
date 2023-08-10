//! # THIR
//!
//! THIR is a typed version of HIR.

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

use ry_ast::{IdentifierAST, Literal, Path, Visibility, WherePredicate};
use ry_filesystem::location::Location;
use ry_fx_hash::FxHashMap;
use ry_interner::IdentifierID;
use ty::Type;

pub mod ty;

/// A pattern, e.g. `Some(x)`, `None`, `a @ [3, ..]`, `[1, .., 3]`, `(1, \"hello\")`, `3.2`.
#[derive(Debug, PartialEq, Clone)]
pub enum Pattern {
    /// A literal pattern, e.g. `3.14`, `'a'`, `true`.
    Literal { literal: Literal, ty: Type },

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
            Self::Literal { literal, .. } => literal.location(),
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
    Rest { location: Location },
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
    Literal { literal: ry_ast::Literal, ty: Type },

    /// Variable expression, e.g. `foo`.
    Variable { name: IdentifierAST, ty: Type },

    /// If expression, e.g. `if x { ... } else { ... }`.
    If {
        location: Location,
        if_blocks: Vec<(Self, Vec<Statement>)>,
        r#else: Option<Vec<Statement>>,
    },

    /// Field access expression, e.g. `x.y`.
    FieldAccess {
        location: Location,
        left: Box<Self>,
        right: IdentifierAST,
    },

    /// Prefix expression, e.g. `!false`, `++a`.
    Prefix {
        location: Location,
        inner: Box<Self>,
        operator: ry_ast::PrefixOperator,
    },

    /// Postfix expression, e.g. `safe_div(1, 0)?`, `a++`.
    Postfix {
        location: Location,
        inner: Box<Self>,
        operator: ry_ast::PostfixOperator,
    },

    /// While expression, e.g. `while x != 0 {}`.
    While {
        location: Location,
        condition: Box<Self>,
        statements_block: Vec<Statement>,
    },

    /// Call expression, e.g. `s.to_string()`.
    Call {
        location: Location,
        callee: Box<Self>,
        type_arguments: Vec<Type>,
        arguments: Vec<Self>,
    },

    /// Static method call, e.g. `String.new()`.
    StaticMethodCall {
        location: Location,
        left: Type,
        right: IdentifierAST,
        type_arguments: Vec<Type>,
        arguments: Vec<Self>,
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
        block: Vec<Statement>,
    },
}

/// A lambda function parameter, e.g. `x` in `|x| { x + 1 }`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LambdaFunctionParameter {
    pub name: IdentifierAST,
    pub ty: Option<Type>,
}

/// A type argument, e.g. `Item = uint32` in `Iterator[Item = uint32]`, `usize` in `sizeof[usize]()`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TypeArgument {
    /// Just a type, e.g. `usize` in `sizeof[usize]()`.
    Type { ty: Type },
    /// Type with a name, e.g. `Item = uint32` in `Iterator[Item = uint32]`.
    AssociatedType { name: IdentifierAST, value: Type },
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
            | Self::Variable {
                name: IdentifierAST { location, .. },
                ..
            }
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
            | Self::StaticMethodCall { location, .. } => *location,
            Self::Literal { literal, .. } => literal.location(),
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
        ty: Type,
    },
}

/// A block of statements - `{ <stmt>* }`.
pub type StatementsBlock = Vec<Statement>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TypeSignature {
    pub name: IdentifierAST,
    pub type_parameters: Vec<IdentifierAST>,
    pub predicates: Vec<WherePredicate>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Enum {
    pub type_signature: TypeSignature,
    pub items: FxHashMap<IdentifierID, EnumItem>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum EnumItem {
    Just(IdentifierAST),
    TupleLike {
        name: IdentifierAST,
        fields: FxHashMap<IdentifierID, EnumItemTupleField>,
    },
    Struct {
        name: IdentifierAST,
        fields: FxHashMap<IdentifierID, EnumItemStructField>,
    },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EnumItemStructField {
    pub location: Location,
    pub ty: Type,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EnumItemTupleField {
    pub location: Location,
    pub ty: Type,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Struct {
    pub type_signature: TypeSignature,
    pub fields: FxHashMap<IdentifierID, StructField>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StructField {
    pub visibility: Visibility,
    pub location: Location,
    pub ty: Type,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TypeAliasSignature {
    pub name: IdentifierAST,
    pub type_parameters: Vec<IdentifierAST>,
    pub value: Type,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InterfaceSignature {
    pub name: IdentifierAST,
    pub type_parameters: Vec<IdentifierAST>,
    pub predicates: Vec<WherePredicate>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FunctionSignature {
    pub name: IdentifierAST,
    pub type_parameters: Vec<IdentifierAST>,
    pub parameters: FxHashMap<IdentifierID, FunctionParameter>,
    pub return_type: Type,
    pub predicates: Vec<WherePredicate>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FunctionParameter {
    pub location: Location,
    pub ty: Type,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Function {
    pub signature: FunctionSignature,
    pub body: Vec<Statement>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Interface {
    pub name: IdentifierAST,
    pub type_parameters: Vec<IdentifierAST>,
    pub predicates: Vec<WherePredicate>,
    pub methods: Vec<Function>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ModuleItemSignature {
    Type(TypeSignature),
    TypeAlias(TypeAliasSignature),
    Interface(InterfaceSignature),
    Function(FunctionSignature),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ModuleItem {
    Enum {
        signature: TypeSignature,
        items: FxHashMap<IdentifierID, EnumItem>,
    },
    Struct {
        signature: TypeSignature,
        fields: FxHashMap<IdentifierID, StructField>,
    },
    Interface {
        signature: InterfaceSignature,
        methods: FxHashMap<IdentifierID, FunctionSignature>,
    },
}
