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

use ry_filesystem::span::Span;
use ry_interner::Symbol;
use token::RawToken;

pub mod precedence;
pub mod serialize;
pub mod token;
pub mod visit;

/// A literal, e.g. `true`, `3`, `\"hello\"`.
#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    /// Boolean literal, e.g. `true` or `false`.
    Boolean { value: bool, span: Span },

    /// Character literal, e.g. `'a'`, `'\u{1234}'`.
    Character { value: char, span: Span },

    /// String literal, e.g. `"hello"`.
    String { value: String, span: Span },

    /// Integer literal, e.g. `123`,
    Integer { value: u64, span: Span },

    /// Float literal, e.g. `3.14`.
    Float { value: f64, span: Span },
}

impl Literal {
    #[inline]
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Self::Boolean { span, .. }
            | Self::Character { span, .. }
            | Self::String { span, .. }
            | Self::Integer { span, .. }
            | Self::Float { span, .. } => *span,
        }
    }
}

/// A symbol with a specified span, e.g. `foo`, `std`.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct IdentifierAst {
    pub span: Span,
    pub symbol: Symbol,
}

/// A sequence of identifiers separated by `.`, e.g. `std.io`, `foo`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Path {
    pub span: Span,
    pub identifiers: Vec<IdentifierAst>,
}

/// An import path, e.g. `std.io`, `std.io as myio`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ImportPath {
    pub left: Path,
    pub r#as: Option<IdentifierAst>,
}

/// A type path, e.g. `Iterator[Item = uint32].Item`, `F.Output`.
#[derive(Debug, PartialEq, Clone)]
pub struct TypePath {
    pub span: Span,
    pub segments: Vec<TypePathSegment>,
}

/// A segment of a type path, e.g. `Iterator[Item = uint32]` and `Item` in
/// `Iterator[Item = uint32].Item`.
#[derive(Debug, PartialEq, Clone)]
pub struct TypePathSegment {
    pub span: Span,
    pub path: Path,
    pub generic_arguments: Option<Vec<GenericArgument>>,
}

/// A pattern, e.g. `Some(x)`, `None`, `a @ [3, ..]`, `[1, .., 3]`, `(1, \"hello\")`, `3.2`.
#[derive(Debug, PartialEq, Clone)]
pub enum Pattern {
    /// A literal pattern, e.g. `3.14`, `'a'`, `true`.
    Literal(Literal),

    /// An identifier pattern, e.g. `f`, `list @ [3, ..]`.
    Identifier {
        span: Span,
        identifier: IdentifierAst,
        pattern: Option<Box<Self>>,
    },

    /// A struct pattern, e.g. `Person { name, age, .. }`.
    Struct {
        span: Span,
        path: Path,
        fields: Vec<StructFieldPattern>,
    },

    /// A tuple-like pattern - used to match a tuple-like structs and enum tuple-like items,
    /// e.g. `Some(x)`, `A()`.
    TupleLike {
        span: Span,
        path: Path,
        inner_patterns: Vec<Self>,
    },

    /// A tuple pattern, e.g. `(a, "hello", ..)`.
    Tuple { span: Span, elements: Vec<Self> },

    /// A path pattern.
    Path { path: Path },

    /// A list pattern, e.g. `[1, .., 10]`.
    List {
        span: Span,
        inner_patterns: Vec<Self>,
    },

    /// A grouped pattern - surrounded by parentheses, e.g. `(a)`, `([1, .., 9])`.
    Grouped { span: Span, inner: Box<Self> },

    /// An or pattern, e.g. `Some(..) | None`.
    Or {
        span: Span,
        left: Box<Self>,
        right: Box<Self>,
    },

    /// A rest pattern - `..`.
    Rest { span: Span },
}

impl Pattern {
    /// Returns the span of the pattern.
    #[inline]
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Self::Literal(
                Literal::Boolean { span, .. }
                | Literal::Character { span, .. }
                | Literal::String { span, .. }
                | Literal::Integer { span, .. }
                | Literal::Float { span, .. },
            )
            | Self::Grouped { span, .. }
            | Self::Identifier { span, .. }
            | Self::List { span, .. }
            | Self::Or { span, .. }
            | Self::Rest { span }
            | Self::Struct { span, .. }
            | Self::Tuple { span, .. }
            | Self::TupleLike { span, .. }
            | Self::Path {
                path: Path { span, .. },
            } => *span,
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
        span: Span,
        field_name: IdentifierAst,
        value_pattern: Option<Pattern>,
    },
    /// A rest pattern, e.g. `..`.
    Rest { span: Span },
}

/// A list of trait bounds being type pathes, e.g. `Debug + Into[T]`.
pub type TypeBounds = Vec<TypePath>;

/// A type, e.g. `int32`, `[S, dyn Iterator[Item = uint32]]`, `(char, char)`.
#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    /// A type path, e.g. `Iterator[Item = uint32].Item`, `R.Output`, `char`.
    Path(TypePath),

    /// A tuple type, e.g. `(int32, String, char)`.
    Tuple {
        span: Span,
        element_types: Vec<Self>,
    },

    /// A function type (return type is required for consistency), e.g. `(char): bool`.
    Function {
        span: Span,
        parameter_types: Vec<Self>,
        return_type: Box<Self>,
    },

    /// A parenthesized type, e.g. `(int32)`.
    ///
    /// **Note**: parenthesized type is not a single element tuple type, because
    /// its syntax is: `(T,)`!
    Parenthesized { span: Span, inner: Box<Self> },

    /// A trait object type, e.g. `dyn Iterator[Item = uint32]`, `dyn Debug + Clone`.
    TraitObject { span: Span, bounds: TypeBounds },

    /// A type with a qualified path, e.g. `[A as Iterator].Item`.
    WithQualifiedPath {
        span: Span,
        left: Box<Self>,
        right: TypePath,
        segments: Vec<TypePathSegment>,
    },
}

impl Type {
    /// Returns the span of the type.
    #[inline]
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Self::Function { span, .. }
            | Self::Parenthesized { span, .. }
            | Self::Path(TypePath { span, .. })
            | Self::TraitObject { span, .. }
            | Self::Tuple { span, .. }
            | Self::WithQualifiedPath { span, .. } => *span,
        }
    }
}

/// A generic parameter, e.g. `T` in `fun into[T](a: T);`.
#[derive(Debug, PartialEq, Clone)]
pub struct GenericParameter {
    pub name: IdentifierAst,
    pub bounds: Option<TypeBounds>,
    pub default_value: Option<Type>,
}

/// A where clause, e.g. `where T: Into<String>, [T as Iterator].Item = char`.
pub type WhereClause = Vec<WhereClauseItem>;

/// A type alias, e.g. `type MyResult = Result[String, MyError]`.
#[derive(Debug, PartialEq, Clone)]
pub struct TypeAlias {
    pub visibility: Visibility,
    pub name: IdentifierAst,
    pub generic_parameters: Option<Vec<GenericParameter>>,
    pub bounds: Option<TypeBounds>,
    pub value: Option<Type>,
    pub docstring: Option<String>,
}

/// A where clause item, e.g. `T: Into<String>` and `[T as Iterator].Item = char` in
/// `where T: Into<String>, [T as Iterator].Item = char`.
#[derive(Debug, PartialEq, Clone)]
pub enum WhereClauseItem {
    Eq { left: Type, right: Type },
    Satisfies { ty: Type, bounds: TypeBounds },
}

/// An expression.
#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    /// List expression, e.g. `[1, 2, 3]`.
    List { span: Span, elements: Vec<Self> },

    /// As expression, e.g. `a as float32`.
    As {
        span: Span,
        left: Box<Self>,
        right: Type,
    },

    /// Binary expression, e.g. `1 + 2`.
    Binary {
        span: Span,
        left: Box<Self>,
        operator: BinaryOperator,
        right: Box<Self>,
    },

    /// Block expression, e.g. `{ let b = 1; b }`.
    StatementsBlock { span: Span, block: Vec<Statement> },

    /// Literal expression, e.g. `true`, `\"hello\"`, `1.2`.
    Literal(Literal),

    /// Identifier expression, e.g. `foo`.
    Identifier(IdentifierAst),

    /// Parenthesized expression, e.g. `(1 + 2)`.
    Parenthesized { span: Span, inner: Box<Self> },

    /// If expression, e.g. `if x { ... } else { ... }`.
    If {
        span: Span,
        if_blocks: Vec<(Self, Vec<Statement>)>,
        r#else: Option<Vec<Statement>>,
    },

    /// Field access expression, e.g. `x.y`.
    FieldAccess {
        span: Span,
        left: Box<Self>,
        right: IdentifierAst,
    },

    /// Prefix expression, e.g. `!false`, `++a`.
    Prefix {
        span: Span,
        inner: Box<Self>,
        operator: PrefixOperator,
    },

    /// Postfix expression, e.g. `safe_div(1, 0)?`, `a++`.
    Postfix {
        span: Span,
        inner: Box<Self>,
        operator: PostfixOperator,
    },

    /// While expression, e.g. `while x != 0 {}`.
    While {
        span: Span,
        condition: Box<Self>,
        body: Vec<Statement>,
    },

    /// Call expression, e.g. `s.to_string()`.
    Call {
        span: Span,
        left: Box<Self>,
        arguments: Vec<Self>,
    },

    /// Generic arguments expression, e.g. `sizeof[uint32]`.
    GenericArguments {
        span: Span,
        left: Box<Self>,
        generic_arguments: Vec<GenericArgument>,
    },

    /// Tuple expression, e.g. `(a, 32, \"hello\")`.
    Tuple { span: Span, elements: Vec<Self> },

    /// Struct expression, e.g. `Person { name: \"John\", age: 25 }`.
    Struct {
        span: Span,
        left: Box<Self>,
        fields: Vec<StructExpressionItem>,
    },

    /// Match expression (`match fs.read_file(...) { ... }`).
    Match {
        span: Span,
        expression: Box<Self>,
        block: Vec<MatchExpressionItem>,
    },

    /// Lambda expression (`|x| { x + 1 }`).
    Lambda {
        span: Span,
        parameters: Vec<LambdaFunctionParameter>,
        return_type: Option<Type>,
        block: Vec<Statement>,
    },
}

/// A lambda function parameter, e.g. `x` in `|x| { x + 1 }`.
#[derive(Debug, Clone, PartialEq)]
pub struct LambdaFunctionParameter {
    pub name: IdentifierAst,
    pub ty: Option<Type>,
}

/// A generic argument, e.g. `Item = uint32` in `Iterator[Item = uint32]`, `usize` in `sizeof[usize]()`.
#[derive(Debug, PartialEq, Clone)]
pub enum GenericArgument {
    /// Just a type, e.g. `usize` in `sizeof[usize]()`.
    Type(Type),
    /// Type with a name, e.g. `Item = uint32` in `Iterator[Item = uint32]`.
    AssociatedType { name: IdentifierAst, value: Type },
}

impl Expression {
    /// Returns the span of the expression.
    #[inline]
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Self::List { span, .. }
            | Self::As { span, .. }
            | Self::Binary { span, .. }
            | Self::StatementsBlock { span, .. }
            | Self::Literal(
                Literal::Integer { span, .. }
                | Literal::Float { span, .. }
                | Literal::Character { span, .. }
                | Literal::String { span, .. }
                | Literal::Boolean { span, .. },
            )
            | Self::Identifier(IdentifierAst { span, .. })
            | Self::Parenthesized { span, .. }
            | Self::If { span, .. }
            | Self::FieldAccess { span, .. }
            | Self::Prefix { span, .. }
            | Self::Postfix { span, .. }
            | Self::While { span, .. }
            | Self::Call { span, .. }
            | Self::GenericArguments { span, .. }
            | Self::Tuple { span, .. }
            | Self::Struct { span, .. }
            | Self::Match { span, .. }
            | Self::Lambda { span, .. } => *span,
        }
    }
}

/// A binary operator with a specific span.
///
/// See [`BinaryOperator`] for more information.
#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub struct BinaryOperator {
    pub span: Span,
    pub raw: RawBinaryOperator,
}

/// A binary operator, e.g. `+`, `**`, `/`.
#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub enum RawBinaryOperator {
    /// Plus Equal (`+=`).
    PlusEq,

    /// Plus (`+`).
    Plus,

    /// Minus Equal (`-=`).
    MinusEq,

    /// Minus (`-`).
    Minus,

    /// Double star (`**`).
    DoubleStar,

    /// Star Equal (`*=`).
    StarEq,

    /// Star (`*`).
    Star,

    /// Slash Equal (`/=`).
    SlashEq,

    /// Slash (`/`).
    Slash,

    /// Not Equal (`!=`).
    BangEq,

    /// Right Shift (`>>`).
    RightShift,

    /// Left Shift (`<<`).
    LeftShift,

    /// Less Equal (`<=`).
    LessEq,

    /// Less (`<`).
    Less,

    /// Greater Equal (`>=`).
    GreaterEq,

    /// Greater (`>`).
    Greater,

    /// Double Equal (`==`).
    EqEq,

    /// Equal (`=`).
    Eq,

    /// Or (`|`).
    Or,

    /// And (`&`).
    And,

    /// Double Or (`||`).
    DoubleOr,

    /// Double And (`&&`).
    DoubleAnd,

    /// Or Equal (`|=`).
    OrEq,

    /// And Equal (`&=`).
    AndEq,

    /// Percent (`%`).
    Percent,

    /// Percent Equal (`%=`).
    PercentEq,
}

impl From<RawToken> for RawBinaryOperator {
    fn from(token: RawToken) -> Self {
        match token {
            Token![+=] => Self::PlusEq,
            Token![+] => Self::Plus,
            Token![-=] => Self::MinusEq,
            Token![-] => Self::Minus,
            Token![*=] => Self::StarEq,
            Token![**] => Self::DoubleStar,
            Token![*] => Self::Star,
            Token![/=] => Self::SlashEq,
            Token![/] => Self::Slash,
            Token![!=] => Self::BangEq,
            Token![>>] => Self::RightShift,
            Token![<<] => Self::LeftShift,
            Token![<=] => Self::LessEq,
            Token![<] => Self::Less,
            Token![>=] => Self::GreaterEq,
            Token![>] => Self::Greater,
            Token![==] => Self::EqEq,
            Token![=] => Self::Eq,
            Token![|] => Self::Or,
            Token![&] => Self::And,
            Token![||] => Self::DoubleOr,
            Token![&&] => Self::DoubleAnd,
            Token![|=] => Self::OrEq,
            Token![&=] => Self::AndEq,
            Token![%] => Self::Percent,
            Token![%=] => Self::PercentEq,
            _ => unreachable!(),
        }
    }
}

impl From<RawBinaryOperator> for RawToken {
    fn from(operator: RawBinaryOperator) -> Self {
        match operator {
            RawBinaryOperator::PlusEq => Token![+=],
            RawBinaryOperator::Plus => Token![+],
            RawBinaryOperator::MinusEq => Token![-=],
            RawBinaryOperator::Minus => Token![-],
            RawBinaryOperator::StarEq => Token![*=],
            RawBinaryOperator::DoubleStar => Token![**],
            RawBinaryOperator::Star => Token![*],
            RawBinaryOperator::SlashEq => Token![/=],
            RawBinaryOperator::Slash => Token![/],
            RawBinaryOperator::BangEq => Token![!=],
            RawBinaryOperator::RightShift => Token![>>],
            RawBinaryOperator::LeftShift => Token![<<],
            RawBinaryOperator::LessEq => Token![<=],
            RawBinaryOperator::Less => Token![<],
            RawBinaryOperator::GreaterEq => Token![>=],
            RawBinaryOperator::Greater => Token![>],
            RawBinaryOperator::EqEq => Token![==],
            RawBinaryOperator::Eq => Token![=],
            RawBinaryOperator::Or => Token![|],
            RawBinaryOperator::And => Token![&],
            RawBinaryOperator::DoubleOr => Token![||],
            RawBinaryOperator::DoubleAnd => Token![&&],
            RawBinaryOperator::OrEq => Token![|=],
            RawBinaryOperator::AndEq => Token![&=],
            RawBinaryOperator::Percent => Token![%],
            RawBinaryOperator::PercentEq => Token![%=],
        }
    }
}

impl From<RawBinaryOperator> for String {
    fn from(value: RawBinaryOperator) -> Self {
        RawToken::from(value).into()
    }
}

impl Display for RawBinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        RawToken::from(*self).fmt(f)
    }
}

/// A prefix operator with a specific span.
///
/// See [`PrefixOperator`] for more information.
#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub struct PrefixOperator {
    pub span: Span,
    pub raw: RawPrefixOperator,
}

/// A prefix operator, e.g. `!`, `++`, `-`.
#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub enum RawPrefixOperator {
    /// Bang (`!`).
    Bang,

    /// Not (`~`).
    Not,

    /// Double Plus (`++`).
    DoublePlus,

    /// Double Minus (`--`).
    DoubleMinus,

    /// Plus (`+`).
    Plus,

    /// Minus (`-`).
    Minus,
}

impl From<RawToken> for RawPrefixOperator {
    fn from(token: RawToken) -> Self {
        match token {
            Token![++] => Self::DoublePlus,
            Token![--] => Self::DoubleMinus,
            Token![+] => Self::Plus,
            Token![-] => Self::Minus,
            Token![!] => Self::Bang,
            Token![~] => Self::Not,
            _ => unreachable!(),
        }
    }
}

impl From<RawPrefixOperator> for RawToken {
    fn from(operator: RawPrefixOperator) -> Self {
        match operator {
            RawPrefixOperator::Bang => Token![!],
            RawPrefixOperator::Not => Token![~],
            RawPrefixOperator::DoublePlus => Token![++],
            RawPrefixOperator::DoubleMinus => Token![--],
            RawPrefixOperator::Plus => Token![+],
            RawPrefixOperator::Minus => Token![-],
        }
    }
}

impl From<RawPrefixOperator> for String {
    fn from(value: RawPrefixOperator) -> Self {
        RawToken::from(value).into()
    }
}

impl Display for RawPrefixOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        RawToken::from(*self).fmt(f)
    }
}

/// A postfix operator with a specific span.
///
/// See [`PostfixOperator`] for more information.
#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub struct PostfixOperator {
    pub span: Span,
    pub raw: RawPostfixOperator,
}

/// A postfix operator, e.g. `?`, `++`.
#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub enum RawPostfixOperator {
    /// Question Mark (`?`).
    QuestionMark,

    /// Double Plus (`++`).
    DoublePlus,

    /// Double Minus (`--`).
    DoubleMinus,
}

impl From<RawToken> for RawPostfixOperator {
    fn from(token: RawToken) -> Self {
        match token {
            Token![?] => Self::QuestionMark,
            Token![++] => Self::DoublePlus,
            Token![--] => Self::DoubleMinus,
            _ => unreachable!(),
        }
    }
}

impl From<RawPostfixOperator> for RawToken {
    fn from(operator: RawPostfixOperator) -> Self {
        match operator {
            RawPostfixOperator::QuestionMark => Token![?],
            RawPostfixOperator::DoublePlus => Token![++],
            RawPostfixOperator::DoubleMinus => Token![--],
        }
    }
}

impl From<RawPostfixOperator> for String {
    fn from(value: RawPostfixOperator) -> Self {
        RawToken::from(value).into()
    }
}

impl Display for RawPostfixOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        RawToken::from(*self).fmt(f)
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
    pub name: IdentifierAst,
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
    Break { span: Span },

    /// Continue statement - `continue`;
    Continue { span: Span },

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

/// A type implementation.
#[derive(Debug, Clone, PartialEq)]
pub struct Impl {
    pub generic_parameters: Option<Vec<GenericParameter>>,
    pub ty: Type,
    pub r#trait: Option<Type>,
    pub where_clause: Option<WhereClause>,
    pub items: Vec<TraitItem>,
    pub docstring: Option<String>,
}

/// A module item.
#[derive(Debug, PartialEq, Clone)]
pub enum ModuleItem {
    /// Enum item.
    Enum {
        visibility: Visibility,
        name: IdentifierAst,
        generic_parameters: Option<Vec<GenericParameter>>,
        where_clause: Option<WhereClause>,
        items: Vec<EnumItem>,
        docstring: Option<String>,
    },

    /// Function item.
    ///
    Function(Function),

    /// Import item.
    ///
    Import { path: ImportPath },

    /// Trait item.
    Trait {
        visibility: Visibility,
        name: IdentifierAst,
        generic_parameters: Option<Vec<GenericParameter>>,
        where_clause: Option<WhereClause>,
        items: Vec<TraitItem>,
        docstring: Option<String>,
    },

    /// Impl item.
    Impl(Impl),

    /// Struct item.
    Struct {
        visibility: Visibility,
        name: IdentifierAst,
        generic_parameters: Option<Vec<GenericParameter>>,
        where_clause: Option<WhereClause>,
        fields: Vec<StructField>,
        docstring: Option<String>,
    },

    /// Tuple-like struct item.
    TupleLikeStruct {
        visibility: Visibility,
        name: IdentifierAst,
        generic_parameters: Option<Vec<GenericParameter>>,
        where_clause: Option<WhereClause>,
        fields: Vec<TupleField>,
        docstring: Option<String>,
    },

    /// Type alias item.
    TypeAlias(TypeAlias),
}

/// A kind of module item.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ItemKind {
    Enum,
    Function,
    Import,
    Trait,
    Impl,
    Struct,
    TypeAlias,
}

impl AsRef<str> for ItemKind {
    fn as_ref(&self) -> &str {
        match self {
            Self::Enum => "enum",
            Self::Function => "function",
            Self::Import => "import",
            Self::Trait => "trait",
            Self::Impl => "type implementation",
            Self::Struct => "struct",
            Self::TypeAlias => "type alias",
        }
    }
}

impl ToString for ItemKind {
    fn to_string(&self) -> String {
        self.as_ref().into()
    }
}

/// An enum item, e.g. `None`, `Ok(T)`, `A { b: T }`.
#[derive(Debug, PartialEq, Clone)]
pub enum EnumItem {
    /// Just an identifier, e.g. `None` in `enum Option[T] { Some(T), None }`.
    Just {
        name: IdentifierAst,
        docstring: Option<String>,
    },
    /// A tuple-like enum item, e.g. `None` in `enum Option<T> { Some(T), None }`.
    TupleLike {
        name: IdentifierAst,
        fields: Vec<TupleField>,
        docstring: Option<String>,
    },
    /// A struct item, e.g. `A { b: T }` in `enum B { A { b: T } }`.
    Struct {
        name: IdentifierAst,
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
    pub name: IdentifierAst,
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
    pub name: IdentifierAst,
    pub generic_parameters: Option<Vec<GenericParameter>>,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: Option<Type>,
    pub where_clause: Option<WhereClause>,
    pub docstring: Option<String>,
}

/// A function parameter, e.g. `self`, `self: Self`, `a: uint32`.
#[derive(Debug, PartialEq, Clone)]
pub enum FunctionParameter {
    /// A function parameter that is not `self`.
    Just(JustFunctionParameter),

    /// A self parameter.
    Self_(SelfParameter),
}

/// A self parameter, e.g. `self`, `self: Self`.
#[derive(Debug, PartialEq, Clone)]
pub struct SelfParameter {
    pub self_span: Span,
    pub ty: Option<Type>,
}

/// A function parameter that is not `self`, e.g. `a: uint32`.
#[derive(Debug, PartialEq, Clone)]
pub struct JustFunctionParameter {
    pub name: IdentifierAst,
    pub ty: Type,
}

/// A Ry module.
#[derive(Debug, PartialEq, Clone)]
pub struct Module {
    pub items: Vec<ModuleItem>,
    pub docstring: Option<String>,
}

/// A visibility qualifier - `pub` or nothing (private visibility).
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
