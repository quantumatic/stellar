//! # HIR
//!
//! In Ry, we have 2 abstract structures that represents relations between tokens - AST and HIR.
//! AST in this case is more abstract than HIR, because HIR is a more desugared version of AST,
//! e.g. `loop` expression doesn't exist for the HIR and is reduced into `while true {}` expression.

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

use std::fmt::Display;

use ry_ast::Bounds;
pub use ry_ast::{IdentifierAST, ImportPath, Literal, Path, TypeConstructor, Visibility};
use ry_filesystem::location::Location;
use ry_interner::Symbol;

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
    },

    /// A struct pattern, e.g. `Person { name, age, .. }`.
    Struct {
        location: Location,
        path: Path,
        fields: Vec<StructFieldPattern>,
    },

    /// A tuple-like pattern - used to match a tuple-like structs and enum tuple-like items,
    /// e.g. `Some(x)`, `A()`.
    TupleLike {
        location: Location,
        path: Path,
        inner_patterns: Vec<Self>,
    },

    /// A tuple pattern, e.g. `(a, "hello", ..)`.
    Tuple {
        location: Location,
        elements: Vec<Self>,
    },

    /// A path pattern.
    Path { path: Path },

    /// A list pattern, e.g. `[1, .., 10]`.
    List {
        location: Location,
        inner_patterns: Vec<Self>,
    },

    /// An or pattern, e.g. `Some(..) | None`.
    Or {
        location: Location,
        left: Box<Self>,
        right: Box<Self>,
    },

    /// A rest pattern - `..`.
    Rest { location: Location },
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
    },

    /// A rest pattern, e.g. `..`.
    Rest { location: Location },
}

/// A type, e.g. `int32`, `(char): bool`, `(char, char)`.
#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    /// A type constructor, e.g. `char`, `List[int32]`.
    Constructor(ry_ast::TypeConstructor),

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

    /// An interface object type, e.g. `dyn Iterator[Item = uint32]`, `dyn Debug + Clone`.
    InterfaceObject {
        location: Location,
        bounds: ry_ast::Bounds,
    },
}

impl Type {
    /// Returns the location of the type.
    #[inline]
    #[must_use]
    pub const fn location(&self) -> Location {
        match self {
            Self::Function { location, .. }
            | Self::Constructor(ry_ast::TypeConstructor { location, .. })
            | Self::InterfaceObject { location, .. }
            | Self::Tuple { location, .. } => *location,
        }
    }
}

/// A generic parameter, e.g. `T` in `fun into[T](a: T);`.
#[derive(Debug, PartialEq, Clone)]
pub struct GenericParameter {
    pub name: IdentifierAST,
    pub bounds: Option<ry_ast::Bounds>,
    pub default_value: Option<Type>,
}

/// A type alias, e.g. `type MyResult = Result[String, MyError]`.
#[derive(Debug, PartialEq, Clone)]
pub struct TypeAlias {
    pub visibility: Visibility,
    pub name: IdentifierAST,
    pub generic_parameters: Vec<GenericParameter>,
    pub bounds: Option<ry_ast::Bounds>,
    pub value: Option<Type>,
    pub docstring: Option<String>,
}

/// A where clause item, e.g. `T: ToString`.
#[derive(Debug, PartialEq, Clone)]
pub struct WherePredicate {
    pub ty: Type,
    pub bounds: ry_ast::Bounds,
}

/// An expression.
#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    /// List expression, e.g. `[1, 2, 3]`.
    List {
        location: Location,
        elements: Vec<Self>,
    },

    /// As expression, e.g. `a as float32`.
    As {
        location: Location,
        left: Box<Self>,
        right: Type,
    },

    /// Binary expression, e.g. `1 + 2`.
    Binary {
        location: Location,
        left: Box<Self>,
        operator: ry_ast::BinaryOperator,
        right: Box<Self>,
    },

    /// Block expression, e.g. `{ let b = 1; b }`.
    StatementsBlock {
        location: Location,
        block: Vec<Statement>,
    },

    /// Literal expression, e.g. `true`, `\"hello\"`, `1.2`.
    Literal(Literal),

    /// Identifier expression, e.g. `foo`.
    Identifier(IdentifierAST),

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
        arguments: Vec<Self>,
    },

    TypeArguments {
        location: Location,
        left: Box<Self>,
        type_arguments: Vec<Type>,
    },

    TypeFieldAccess {
        location: Location,
        left: Type,
        right: IdentifierAST,
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
#[derive(Debug, Clone, PartialEq)]
pub struct LambdaFunctionParameter {
    pub name: IdentifierAST,
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
        ty: Option<Type>,
    },
}

/// A block of statements - `{ <stmt>* }`.
pub type StatementsBlock = Vec<Statement>;

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
    pub generic_parameters: Vec<GenericParameter>,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: Option<Type>,
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
        generic_parameters: Vec<GenericParameter>,
        where_predicates: Vec<WherePredicate>,
        items: Vec<EnumItem>,
        methods: Vec<Function>,
        implements: Option<Bounds>,
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

    /// Interface item.
    Interface {
        visibility: Visibility,
        name: IdentifierAST,
        generic_parameters: Vec<GenericParameter>,
        where_predicates: Vec<WherePredicate>,
        methods: Vec<Function>,
        implements: Option<Bounds>,
        docstring: Option<String>,
    },

    /// Struct item.
    Struct {
        visibility: Visibility,
        name: IdentifierAST,
        generic_parameters: Vec<GenericParameter>,
        where_predicates: Vec<WherePredicate>,
        fields: Vec<StructField>,
        methods: Vec<Function>,
        implements: Option<Bounds>,
        docstring: Option<String>,
    },

    /// Tuple-like struct item.
    TupleLikeStruct {
        visibility: Visibility,
        name: IdentifierAST,
        generic_parameters: Vec<GenericParameter>,
        where_predicates: Vec<WherePredicate>,
        fields: Vec<TupleField>,
        methods: Vec<Function>,
        implements: Option<Bounds>,
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
            | Self::Import { location, .. }
            | Self::Struct {
                name: IdentifierAST { location, .. },
                ..
            }
            | Self::Interface {
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
            | Self::Interface {
                name: IdentifierAST { symbol, .. },
                ..
            }
            | Self::TypeAlias(TypeAlias {
                name: IdentifierAST { symbol, .. },
                ..
            }) => Some(*symbol),
            Self::Import { .. } => None,
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
            Self::Enum { visibility, .. }
            | Self::Struct { visibility, .. }
            | Self::TupleLikeStruct { visibility, .. }
            | Self::Interface { visibility, .. }
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
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ModuleItemKind {
    Enum,
    Function,
    Import,
    Interface,
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
            Self::Interface => "interface",
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
    pub ty: Type,
}

/// A struct field, e.g. `name: String`, `pub age: uint32`.
#[derive(Debug, PartialEq, Clone)]
pub struct StructField {
    pub visibility: Visibility,
    pub name: IdentifierAST,
    pub ty: Type,
    pub docstring: Option<String>,
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
    pub ty: Option<Type>,
}

/// A function parameter that is not `self`, e.g. `a: uint32`.
#[derive(Debug, PartialEq, Clone)]
pub struct NotSelfFunctionParameter {
    pub name: IdentifierAST,
    pub ty: Type,
}

/// A Ry module.
#[derive(Debug, PartialEq, Clone)]
pub struct Module {
    pub items: Vec<ModuleItem>,
    pub docstring: Option<String>,
}
