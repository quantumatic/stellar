//! # Typed AST
//!
//! Typed AST is a representation of a Ry program, that is produced
//! after the process called type inference.
//!
//! Typed AST is similar to the AST, but with type annotations.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png",
    html_favicon_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png"
)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
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

use ry_ast::{BinaryOperator, Docstring, PostfixOperator, PrefixOperator};
use ry_filesystem::span::Span;
use ry_interner::Symbol;
use ty::Type;

pub mod ty;

/// Represents a literal.
#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Boolean(bool),
    Character(char),
    String(String),
    Integer(u64),
    Float(f64),
}

/// Fully qualified path.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Path {
    /// Symbols in the path.
    pub symbols: Vec<Symbol>,
}

/// Represents a pattern AST node.
///
/// # Example
///
/// Here is an example of it is used in the match expression:
/// ```txt
/// match x {
///     Some(a) => { println(a); }
///     ^^^^^^^ pattern
///     None => { panic("something went wrong"); }
///     ^^^^ pattern
/// }
/// ```
#[derive(Debug, PartialEq, Clone)]
pub enum Pattern {
    /// A literal pattern.
    ///
    /// # Example
    ///
    /// ```txt
    /// match x {
    ///     3 => { println("x is 3!"); }
    ///     ^ literal pattern
    ///     .. => { println("x is not 3!"); }
    /// }
    /// ```
    Literal(Literal),

    /// An identifier pattern.
    ///
    /// Used to store a value corresponding to some pattern.
    ///
    /// # Example
    /// ```txt
    /// match x {
    ///     [.., b @ [3, ..]] => { println(b); }
    ///          ^^^^^^^^^^^ identifier pattern
    ///     .. => { println(":("); }
    /// }
    /// ```
    /// In the example, `b` is now having a value corresponding to the pattern `[3, ..]`.
    Identifier {
        identifier: Symbol,
        pattern: Option<Box<Self>>,
    },

    /// A struct pattern.
    ///
    /// # Example
    /// ```txt
    /// match person {
    ///     Person { citizenship: "USA" } => { println("Welcome to your homeland, comrade!"); }
    ///     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ struct pattern
    ///
    ///     .. => { println("Welcome to the USA!"); }
    /// }
    /// ```
    Struct {
        path: Path,
        fields: Vec<StructFieldPattern>,
    },

    /// A tuple-like pattern.
    /// Used to match a tuple-like structs and enum tuple-like items.
    ///
    /// # Example
    /// ```txt
    /// match x {
    ///     Some(x) => { println(x); }
    ///     ^^^^^^^ tuple-like pattern
    ///
    ///     None => { panic("something went wrong"); }
    ///     ^^^^ path pattern
    /// }
    /// ```
    TupleLike {
        path: Path,
        inner_patterns: Vec<Self>,
    },

    /// A tuple pattern. Used to match tuple expressions.
    ///
    /// # Example
    /// ```txt
    /// match x {
    ///     (a, "hello", c @ [3, ..]) => { println(a); }
    ///     ^^^^^^^^^^^^^^^^^^^^^^^^^^ tuple pattern
    ///
    ///     .. => { println(":("); }
    /// }
    /// ```
    Tuple { elements: Vec<Self> },

    /// A path pattern.
    ///
    /// # Examples
    /// Path pattern with single identifier in it (do not mess it with
    /// tuple-like or struct patterns):
    /// ```txt
    /// match x {
    ///     Some(a) => { println(a); }
    ///     ^^^^^^^ tuple-like pattern
    ///     None => { println("none"); }
    ///     ^^^^ path pattern
    /// }
    /// ```
    ///
    /// Path pattern with multiple identifiers in it:
    /// ```txt
    /// match x {
    ///     module.x => { println("x == module.x"); }
    ///     ^^^^^^^^ path pattern
    ///
    ///     .. => { println("x != module.x"); }
    /// }
    /// ```
    Path { path: Path },

    /// A list pattern.
    ///
    /// # Example
    /// ```txt
    ///
    /// match x {
    ///     [.., b @ [3, ..]] => { println(b); }
    ///              ^^^^^^^ list pattern
    ///
    ///     .. => { println(":("); }
    /// }
    /// ```
    List { inner_patterns: Vec<Self> },

    /// A grouped pattern. (just a pattern surrounded by parentheses)
    ///
    /// # Example
    /// ```txt
    ///
    /// match x {
    ///     (Some(..)) => { println("some"); }
    ///     ^^^^^^^^^^ grouped pattern
    ///
    ///     ((None)) => { println("none"); }
    ///     ^^^^^^^^ grouped pattern
    ///      ^^^^^^ grouped pattern inside of the grouped pattern
    /// }
    /// ```
    Grouped { inner: Box<Self> },

    /// An or pattern.
    ///
    /// # Example
    /// ```txt
    ///
    /// match x {
    ///     // always matches
    ///     Some(..) | None => { println("ok"); }
    ///     ^^^^^^^^^^^^^^^ or pattern
    /// }
    /// ```
    Or { left: Box<Self>, right: Box<Self> },

    /// A rest pattern.
    ///
    /// # Example
    /// ```txt
    /// match x {
    ///     // always matches
    ///     .. => { println("ok"); }
    ///     ^^ rest pattern
    /// }
    Rest,
}

/// Represents a pattern used inside of a struct pattern.
///
/// # Example
/// ```txt
/// match person {
///     Person { citizenship: "USA", name, .. } => {
///              ------------------  ---- not rest struct field patterns
///                                        -- rest struct field pattern
///
///        println("Welcome to your homeland " + name + "!");
///     }
///
///     .. => { println("Welcome to the USA!"); }
/// }
/// ```
#[derive(Debug, PartialEq, Clone)]
pub enum StructFieldPattern {
    NotRest {
        field_name: Symbol,
        value_pattern: Option<Pattern>,
    },
    Rest,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypePath {
    pub segments: Vec<TypePathSegment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypePathSegment {
    pub left: Path,
    pub right: Vec<Type>,
}

/// Represents a list of trait bounds being type pathes.
pub type TypeBounds = Vec<Path>;

/// Represents a generic parameter.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GenericParameter {
    pub symbol: Symbol,
    pub bounds: Option<TypeBounds>,
}

/// Represents a where clause.
///
/// ```txt
/// impl[T] ToString for T where T: Into[String] { ... }
///                        ^^^^^^^^^^^^^^^^^^^^^ where clause
/// ```
pub type WhereClause = Vec<WhereClauseItem>;

/// Represents a type alias.
///
/// ```txt
/// type StringRes[E] = Result[String, E];
#[derive(Debug, PartialEq, Clone)]
pub struct TypeAlias {
    pub span: Span,
    pub name: Symbol,
    pub generic_parameters: Option<Vec<GenericParameter>>,
    pub bounds: Option<TypeBounds>,
    pub value: Option<Type>,
    pub docstring: Option<Docstring>,
}

/// Represents a where clause item.
///
/// ```txt
/// impl[T, M] ToString for (T, M) where T: Into[String], M = dyn Into[String] { ... }
///                                       ^^^^^^^^^^^^^^^ where clause item #1
///                                                        ^^^^^^^^^^^^^^^^^^^^ where clause item #2
/// ```
#[derive(Debug, PartialEq, Clone)]
pub enum WhereClauseItem {
    Eq { left: Type, right: Type },
    Satisfies { ty: Type, bounds: TypeBounds },
}

/// Represents an expression in a typed AST.
#[derive(Debug, PartialEq, Clone)]
pub enum TypedExpression {
    /// List expression.
    ///
    /// ```txt
    /// [1, 2, 3]
    /// ```
    List {
        span: Span,
        elements: Vec<Self>,
        ty: Type,
    },

    /// As expression.
    ///
    /// ```txt
    /// 3 as float32
    /// ```
    As {
        span: Span,
        left: Box<Self>,
        right: Type, // == type
    },

    /// Binary expression.
    ///
    /// ```txt
    /// 1 + 2
    /// ```
    Binary {
        span: Span,
        left: Box<Self>,
        operator: BinaryOperator,
        right: Box<Self>,
        ty: Type,
    },

    /// Block expression.
    ///
    /// ```txt
    /// {
    ///     let b = 1;
    ///     b + 2
    /// };
    /// ```
    StatementsBlock {
        span: Span,
        block: Vec<Statement>,
        ty: Type,
    },

    /// Literal expression.
    ///
    /// ```txt
    /// "hello"
    /// ```
    Literal {
        span: Span,
        value: Literal,
        ty: Type,
    },

    /// Identifier expression.
    ///
    /// ```txt
    /// x
    /// ```
    Identifier {
        span: Span,
        symbol: Symbol,
        ty: Type,
    },

    /// If expression.
    ///
    /// ```txt
    /// if x < 2 {
    ///     1
    /// } else {
    ///     factorial(x - 1) * x
    /// }
    /// ```
    If {
        span: Span,
        if_blocks: Vec<(Self, Vec<Statement>)>,
        r#else: Option<Vec<Statement>>,
        ty: Type,
    },

    /// Property expression.
    ///
    /// ```txt
    /// x.y
    /// ```
    FieldAccess {
        span: Span,
        left: Box<Self>,
        right: Symbol,
    },

    /// Type path call.
    ///
    /// ```txt
    /// uint32.max()
    /// ```
    TypePathCall {
        span: Span,
        left: Box<Self>,
        arguments: Vec<Self>,
        ty: Type,
    },

    /// Call expression.
    ///
    /// ```txt
    /// s.to_string()
    /// ```
    Call {
        span: Span,
        left: Box<Self>,
        function: Symbol,
        arguments: Vec<Self>,
        ty: Type,
    },

    /// Prefix expression.
    ///
    /// ```txt
    /// !x
    /// ```
    Prefix {
        span: Span,
        inner: Box<Self>,
        operator: PrefixOperator,
        ty: Type,
    },

    /// Postfix expression.
    ///
    /// ```txt
    /// returns_option()?
    /// ```
    Postfix {
        span: Span,
        inner: Box<Self>,
        operator: PostfixOperator,
        ty: Type,
    },

    /// While expression (always returns `Unit` type).
    ///
    /// ```txt
    /// while x < 2 {
    ///     break;
    /// }
    /// ```
    While {
        span: Span,
        condition: Box<Self>,
        body: Vec<Statement>,
        ty: Type,
    },

    /// Tuple expression.
    ///
    /// ```txt
    /// (a, "hello", 3)
    /// (a,)
    /// ```
    Tuple {
        span: Span,
        elements: Vec<Self>,
        ty: Type,
    },

    /// Struct expression.
    ///
    /// ```txt
    /// let person = Person { name: "John", age: 30 };
    ///              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ struct expression
    /// ```
    Struct {
        span: Span,
        left: Box<Self>,
        fields: Vec<StructExpressionItem>,
        ty: Type,
    },

    /// Match expression.
    ///
    /// ```txt
    /// match fs.read_file("foo.txt") {
    ///     Ok(data) => { println(data); },
    ///     Err(e) => { println("something went wrong"); }
    /// }
    /// ```
    Match {
        span: Span,
        expression: Box<Self>,
        block: Vec<MatchExpressionItem>,
    },

    /// Function expression.
    ///
    /// ```txt
    /// let a = |x: uint32|: uint32 { x + 1 };
    ///         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ function expression
    /// ```
    Function {
        span: Span,
        parameters: Vec<JustFunctionParameter>,
        return_type: Type,
        block: Vec<Statement>,
    },
}

/// Represents a generic argument.
#[derive(Debug, PartialEq, Clone)]
pub enum GenericArgument {
    Type(Type),
    AssociatedType { name: Symbol, value: Type },
}

/// Represents a match expression item (`pattern` `=>` `expression`).
///
/// ```txt
/// match 1.safe_div(0) {
///    Some(x) => x,
///    ^^^^^^^^^^^^ match expression item
///
///    None => { panic("you can't divide by zero") },
///    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ match expression item
/// }
#[derive(Debug, PartialEq, Clone)]
pub struct MatchExpressionItem {
    pub left: Pattern,
    pub right: TypedExpression,
}

/// Represents a field initialization in a struct expression (`identifier` and optionally `:` `expression`).
///
/// ```txt
/// let age = 30;
///
/// let person = Person {
///     name: "John",
///     ^^^^^^^^^^^^ struct expression item
///     age,
///     ^^^ struct expression item
/// }
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct StructExpressionItem {
    pub name: Symbol,
    pub value: Option<TypedExpression>,
}

/// Represents a statement.
#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    /// Defer statement
    ///
    /// ```txt
    /// defer file.close();
    /// ```
    Defer { call: TypedExpression },

    /// Expression statement
    ///
    /// ```txt
    /// if x {
    ///     return Some("hello");
    /// }
    /// ```
    Expression {
        expression: TypedExpression,
        has_semicolon: bool,
    },

    /// Break statement
    ///
    /// ```txt
    /// break;
    /// ```
    Break,

    /// Continue statement
    ///
    /// ```txt
    /// continue;
    /// ```
    Continue,

    /// Return statement
    ///
    /// ```txt
    /// /// Answer to the Ultimate Question of Life, the Universe, and Everything
    /// fun the_answer(): uint32 {
    ///     return 42;
    /// }
    /// ```
    Return { expression: TypedExpression },

    /// Let statement
    ///
    /// ```txt
    /// let x = 1;
    /// ```
    Let {
        pattern: Pattern,
        value: Box<TypedExpression>,
        ty: Type,
    },
}

/// Represents a block of statements.
///
/// ```txt
/// fun main() { println!("Hello"); }
///            ^^^^^^^^^^^^^^^^^^^^^^ statements block
/// ```
pub type StatementsBlock = Vec<Statement>;

/// Represents an item.
#[derive(Debug, PartialEq, Clone)]
pub enum Item {
    /// Enum item.
    ///
    /// ```txt
    /// enum UserCredentials {
    ///     None,
    ///     EmailOnly(String)
    ///     PhoneNumberOnly(String)
    ///     PhoneAndEmail {
    ///         phone: String,
    ///         email: String
    ///     }
    /// }
    /// ```
    Enum {
        span: Span,
        name: Symbol,
        generic_parameters: Option<Vec<GenericParameter>>,
        where_clause: Option<WhereClause>,
        items: Vec<EnumItem>,
        docstring: Option<Docstring>,
    },

    /// Function item.
    ///
    /// ```txt
    /// fun foo() {
    ///     println("Hello")
    /// }
    /// ```
    Function(Function),

    /// Trait item.
    ///
    /// ```txt
    /// trait Into[T] {
    ///     fun into(self: Self) -> T;
    /// }
    /// ```
    Trait {
        span: Span,
        name: Symbol,
        generic_parameters: Option<Vec<GenericParameter>>,
        where_clause: Option<WhereClause>,
        items: Vec<TraitItem>,
        docstring: Option<Docstring>,
    },

    /// Impl item.
    ///
    /// ```txt
    /// impl Person {
    ///     pub fun new(name: String) -> Self {
    ///         Self {
    ///             name
    ///         }
    ///     }
    /// }
    /// ```
    Impl {
        generic_parameters: Option<Vec<GenericParameter>>,
        ty: Type,
        r#trait: Option<TypePath>,
        where_clause: Option<WhereClause>,
        items: Vec<TraitItem>,
        docstring: Option<Docstring>,
    },

    /// Struct item.
    ///
    /// ```txt
    /// struct Person {
    ///     name: String,
    ///     age: uint32,
    ///     citizenship: String
    /// }
    /// ```
    Struct {
        visibility: Visibility,
        span: Span,
        name: Symbol,
        generic_parameters: Option<Vec<GenericParameter>>,
        where_clause: Option<WhereClause>,
        fields: Vec<StructField>,
        docstring: Option<Docstring>,
    },

    /// Tuple-like struct item.
    ///
    /// ```txt
    /// struct MyStringWrapper(String);
    /// ```
    TupleLikeStruct {
        visibility: Visibility,
        span: Span,
        name: Symbol,
        generic_parameters: Option<Vec<GenericParameter>>,
        where_clause: Option<WhereClause>,
        fields_visibility: Visibility,
        fields: Vec<Type>,
        docstring: Option<Docstring>,
    },

    /// Type alias item.
    TypeAlias(TypeAlias),
}

/// Represents an enum item.
///
/// ```txt
/// enum UserCredentials {
///     None,
///     ^^^^ enum item
///     EmailOnly(String),
///     ^^^^^^^^^^^^^^^^^ enum item
///     PhoneNumberOnly(String),
///     ^^^^^^^^^^^^^^^^^^^^^^^ enum item
///
///     ...
/// }
/// ```
#[derive(Debug, PartialEq, Clone)]
pub enum EnumItem {
    Just {
        span: Span,
        name: Symbol,
        docstring: Option<Docstring>,
    },
    Tuple {
        span: Span,
        name: Symbol,
        fields: Vec<Type>,
        docstring: Option<Docstring>,
    },
    Struct {
        span: Span,
        name: Symbol,
        fields: Vec<StructField>,
        docstring: Option<Docstring>,
    },
}

/// Represents a struct field.
///
/// ```txt
/// struct Person {
///     name: String,
///     ^^^^^^^^^^^^ struct field
///     age: uint32
///     ^^^^^^^^^^^ struct field
/// }
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct StructField {
    pub span: Span,
    pub visibility: Visibility,
    pub name: Symbol,
    pub ty: Type,
    pub docstring: Option<Docstring>,
}

/// Represents a trait item.
#[derive(Debug, PartialEq, Clone)]
pub enum TraitItem {
    TypeAlias(TypeAlias),
    AssociatedFunction(Function),
}

/// Represents a function.
///
/// ```txt
/// fun sum[T](a: T, b: T) -> T where T: Add[T, T] { a + b }
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct Function {
    pub span: Span,
    pub name: Symbol,
    pub generic_parameters: Option<Vec<GenericParameter>>,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: Type,
    pub where_clause: Option<WhereClause>,
    pub body: Option<StatementsBlock>,
    pub docstring: Option<Docstring>,
}

/// Represents a function parameter.
#[derive(Debug, PartialEq, Clone)]
pub enum FunctionParameter {
    Just(JustFunctionParameter),
    Self_(SelfParameter),
}

/// Represents a self parameter.
///
/// ```txt
/// fun to_string(self) -> String {
///               ^^^^
/// }
#[derive(Debug, PartialEq, Clone)]
pub struct SelfParameter {
    pub self_span: Span,
    pub ty: Option<Type>,
}

/// Represents a function parameter that is not `self`.
///
/// ```txt
/// pub fun sum[T](a: T, b: T) -> T where T: Add[T, T] {
///                ^^^^  ^^^^
///     a + b
/// }
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct JustFunctionParameter {
    pub name: Symbol,
    pub ty: Type,
}

/// Represents Ry source file.
#[derive(Debug, PartialEq, Clone)]
pub struct Module {
    pub items: Vec<Item>,
    pub docstring: Option<Docstring>,
}

/// Represents a visibility qualifier.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Visibility(Option<Span>);

impl Visibility {
    #[inline]
    #[must_use]
    pub const fn private() -> Self {
        Self(None)
    }

    #[inline]
    #[must_use]
    pub const fn public(span: Span) -> Self {
        Self(Some(span))
    }

    #[inline]
    #[must_use]
    pub const fn span_of_pub(&self) -> Option<Span> {
        self.0
    }
}

impl Default for Visibility {
    fn default() -> Self {
        Self::private()
    }
}
