//! `lib.rs` - defines AST nodes and additional stuff.
#![warn(
    clippy::all,
    clippy::doc_markdown,
    clippy::dbg_macro,
    clippy::todo,
    clippy::mem_forget,
    clippy::filter_map_next,
    clippy::needless_continue,
    clippy::needless_borrow,
    clippy::match_wildcard_for_single_variants,
    clippy::mismatched_target_os,
    clippy::match_on_vec_items,
    clippy::imprecise_flops,
    clippy::suboptimal_flops,
    clippy::lossy_float_literal,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::fn_params_excessive_bools,
    clippy::inefficient_to_string,
    clippy::linkedlist,
    clippy::macro_use_imports,
    clippy::option_option,
    clippy::verbose_file_reads,
    rust_2018_idioms,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    nonstandard_style,
    unused_import_braces,
    unused_qualifications
)]
#![deny(
    clippy::await_holding_lock,
    clippy::if_let_mutex,
    clippy::indexing_slicing,
    clippy::mem_forget,
    clippy::ok_expect,
    clippy::unimplemented,
    clippy::unwrap_used,
    unsafe_code,
    unstable_features,
    unused_results
)]
#![allow(clippy::match_single_binding, clippy::inconsistent_struct_constructor)]

use ry_interner::Symbol;
use ry_source_file::{source_file::SourceFile, span::Span, workspace::FileID};
use token::RawToken;

pub mod precedence;
pub mod serialize;
pub mod token;
pub mod typed;

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Boolean { value: bool, span: Span },
    Character { value: char, span: Span },
    String { value: String, span: Span },
    Integer { value: u64, span: Span },
    Float { value: f64, span: Span },
}

/// Symbol with a specified span.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct IdentifierAst {
    pub span: Span,
    pub symbol: Symbol,
}

/// AST Node corresponding to a sequence of identifiers separated by `.`.
///
/// # Example
/// Here is an example of it is used in the use item:
///
/// ```txt
/// use std.io;
///     ^^^^^^
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct Path {
    pub span: Span,
    pub identifiers: Vec<IdentifierAst>,
}

/// AST Node for a type path.
///
/// ```txt
/// let a: Iterator[Item = uint32].Item = 3;
///        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct TypePath {
    pub span: Span,
    pub segments: Vec<TypePathSegment>,
}

/// A type path segment.
///
/// ```txt
/// let a: Iterator[Item = uint32].Item = 3;
///        ^^^^^^^^^^^^^^^^^^^^^^^ ^^^^
#[derive(Debug, PartialEq, Clone)]
pub struct TypePathSegment {
    pub span: Span,
    pub path: Path,
    pub generic_arguments: Option<Vec<GenericArgument>>,
}

/// AST Node for a pattern.
///
/// # Example
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
        span: Span,
        identifier: IdentifierAst,
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
        span: Span,
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
        span: Span,
        path: Path,
        inner_patterns: Vec<Self>,
    },

    /// A tuple pattern. Used to match tuple expressions.
    ///
    /// # Example
    /// ```txt
    /// match x {
    ///     #(a, "hello", c @ [3, ..]) => { println(a); }
    ///     ^^^^^^^^^^^^^^^^^^^^^^^^^^ tuple pattern
    ///
    ///     .. => { println(":("); }
    /// }
    /// ```
    Tuple {
        span: Span,
        inner_patterns: Vec<Self>,
    },

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
    Path { span: Span, path: Path },

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
    List {
        span: Span,
        inner_patterns: Vec<Self>,
    },

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
    Grouped { span: Span, inner: Box<Self> },

    /// An or pattern.
    ///
    /// # Example
    /// ```txt
    ///
    /// match x {
    ///     // always matches
    ///     Some(..) | None => { println("ok"); }
    ///     ^^^^^^^^^^^^^^^ or pattern
    ///
    ///     _ => { unreachable(); }
    /// }
    /// ```
    Or {
        span: Span,
        left: Box<Self>,
        right: Box<Self>,
    },

    /// A rest pattern.
    ///
    /// # Example
    /// ```txt
    /// match x {
    ///     // always matches
    ///     .. => { println("ok"); }
    ///     ^^ rest pattern
    ///
    ///     _ => { unreachable(); }
    /// }
    Rest { span: Span },
}

impl Pattern {
    /// Returns the span of the pattern.
    pub fn span(&self) -> Span {
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
            | Self::Path { span, .. } => *span,
        }
    }
}

/// A pattern used inside a struct pattern.
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
        span: Span,
        field_name: IdentifierAst,
        value_pattern: Option<Pattern>,
    },
    Rest {
        span: Span,
    },
}

/// An AST node used to represent type bounds.
///
/// # Example
/// ```txt
/// pub trait DefaultAndDebug: Default + Debug {
///                            ^^^^^^^^^^^^^^^
/// }
/// ```
pub type TypeBounds = Vec<TypePath>;

/// An AST node used to represent types in an untyped AST.
///
/// # Example
/// ```txt
/// let a: #(List[uint32], String, uint32) = #(a, "hello", 3);
///        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ type node
/// ```
#[derive(Debug, PartialEq, Clone)]
pub enum TypeAst {
    /// A type path.
    ///
    /// ```txt
    /// String
    /// uint32
    /// Iterator[Item = uint32].Item
    /// List[uint32]
    /// ```
    Path(TypePath),

    /// A tuple type.
    ///
    /// ```txt
    /// #(String, uint32)
    /// ```
    Tuple {
        span: Span,
        element_types: Vec<Self>,
    },

    /// A function type (return type is required for consistency).
    ///
    /// ```txt
    /// (uint32) -> Unit
    /// ```
    Function {
        span: Span,
        parameter_types: Vec<Self>,
        return_type: Box<Self>,
    },

    /// A parenthesized type.
    ///
    /// ```txt
    /// (Option[uint32])
    /// ```
    Parenthesized { span: Span, inner: Box<Self> },

    /// A trait object type.
    ///
    /// ```txt
    /// dyn Iterator[Item = uint32]
    /// ```
    TraitObject { span: Span, bounds: TypeBounds },

    /// A type with a qualified path.
    ///
    /// ```txt
    /// [List[[List[uint32] as IntoIterator].Item] as IntoIterator].Item
    ///        ^^^^^^^^^^^^    ^^^^^^^^^^^^  ^^^^
    ///        |               |             |
    ///        left            right         segments[0]
    /// ```
    WithQualifiedPath {
        span: Span,
        left: Box<Self>,
        right: TypePath,
        segments: Vec<TypePathSegment>,
    },
}

impl TypeAst {
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

/// A generic parameter.
///
/// # Example
///
/// ```txt
/// fun into[T](a: T) { ... }
///          ^ generic parameter
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct GenericParameter {
    pub name: IdentifierAst,
    pub bounds: Option<TypeBounds>,
    pub default_value: Option<TypeAst>,
}

/// A where clause.
///
/// ```txt
/// impl[T] ToString for T where T: Into[String] { ... }
///                        ^^^^^^^^^^^^^^^^^^^^^ where clause
/// ```
pub type WhereClause = Vec<WhereClauseItem>;

/// A type alias.
///
/// ```txt
/// type StringRes[E] = Result[String, E];
#[derive(Debug, PartialEq, Clone)]
pub struct TypeAlias {
    pub visibility: Visibility,
    pub name: IdentifierAst,
    pub generic_parameters: Option<Vec<GenericParameter>>,
    pub bounds: Option<TypeBounds>,
    pub value: Option<TypeAst>,
}

/// A where clause item.
///
/// ```txt
/// impl[T, M] ToString for #(T, M) where T: Into[String], M = dyn Into[String] { ... }
///                                       ^^^^^^^^^^^^^^^ where clause item #1
///                                                        ^^^^^^^^^^^^^^^^^^^^ where clause item #2
/// ```
#[derive(Debug, PartialEq, Clone)]
pub enum WhereClauseItem {
    Eq { left: TypeAst, right: TypeAst },
    Satisfies { ty: TypeAst, bounds: TypeBounds },
}

/// Represents an expression in an untyped AST.
#[derive(Debug, PartialEq, Clone)]
pub enum UntypedExpression {
    /// List expression.
    ///
    /// ```txt
    /// [1, 2, 3]
    /// ```
    List { span: Span, elements: Vec<Self> },

    /// As expression.
    ///
    /// ```txt
    /// 3 as float32
    /// ```
    As {
        span: Span,
        left: Box<Self>,
        right: TypeAst,
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
    },

    /// Block expression.
    ///
    /// ```txt
    /// {
    ///     let b = 1;
    ///     b + 2
    /// };
    /// ```
    StatementsBlock { span: Span, block: Vec<Statement> },

    /// Literal expression.
    ///
    /// ```txt
    /// "hello"
    /// ```
    Literal(Literal),

    /// Identifier expression.
    ///
    /// ```txt
    /// x
    /// ```
    Identifier(IdentifierAst),

    /// Parenthesized expression.
    ///
    /// ```txt
    /// (1 + 2)
    /// ```
    Parenthesized { span: Span, inner: Box<Self> },

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
    },

    /// Property expression.
    ///
    /// ```txt
    /// x.y
    /// ```
    FieldAccess {
        span: Span,
        left: Box<Self>,
        right: IdentifierAst,
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
    },

    /// Call expression.
    ///
    /// ```txt
    /// s.to_string()
    /// ```
    Call {
        span: Span,
        left: Box<Self>,
        arguments: Vec<Self>,
    },

    /// Generic arguments expression.
    ///
    /// ```txt
    /// into[uint32](3);
    /// ^^^^^^^^^^^^ generic arguments expression
    ///             ^^^ call
    /// ```
    GenericArguments {
        span: Span,
        left: Box<Self>,
        generic_arguments: Vec<GenericArgument>,
    },

    /// Tuple expression.
    ///
    /// ```txt
    /// #(a, "hello", 3)
    /// ```
    Tuple { span: Span, elements: Vec<Self> },

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
        return_type: Option<TypeAst>,
        block: Vec<Statement>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum GenericArgument {
    Type(TypeAst),
    AssociatedType { name: IdentifierAst, value: TypeAst },
}

impl UntypedExpression {
    /// Returns the span of the expression.
    pub fn span(&self) -> Span {
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
            | Self::Function { span, .. } => *span,
        }
    }
}

/// Represents a binary operator with a specific span.
#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub struct BinaryOperator {
    pub span: Span,
    pub raw: RawBinaryOperator,
}

/// Represents a binary operator.
#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub enum RawBinaryOperator {
    PlusEq,
    Plus,
    MinusEq,
    Minus,
    StarStar,
    StarEq,
    Star,
    SlashEq,
    Slash,
    NotEq,
    Bang,
    RightShift,
    LeftShift,
    LessEq,
    Less,
    GreaterEq,
    Greater,
    EqEq,
    Eq,
    Or,
    And,
    OrOr,
    AndAnd,
    OrEq,
    AndEq,
    Percent,
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
            Token![**] => Self::StarStar,
            Token![*] => Self::Star,
            Token![/=] => Self::SlashEq,
            Token![/] => Self::Slash,
            Token![!=] => Self::NotEq,
            Token![!] => Self::Bang,
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
            Token![||] => Self::OrOr,
            Token![&&] => Self::AndAnd,
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
            RawBinaryOperator::StarStar => Token![**],
            RawBinaryOperator::Star => Token![*],
            RawBinaryOperator::SlashEq => Token![/=],
            RawBinaryOperator::Slash => Token![/],
            RawBinaryOperator::NotEq => Token![!=],
            RawBinaryOperator::Bang => Token![!],
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
            RawBinaryOperator::OrOr => Token![||],
            RawBinaryOperator::AndAnd => Token![&&],
            RawBinaryOperator::OrEq => Token![|=],
            RawBinaryOperator::AndEq => Token![&=],
            RawBinaryOperator::Percent => Token![%],
            RawBinaryOperator::PercentEq => Token![%=],
        }
    }
}

/// Represents a prefix operator with a specific span.
#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub struct PrefixOperator {
    pub span: Span,
    pub raw: RawPrefixOperator,
}

/// Represents a prefix operator.
#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub enum RawPrefixOperator {
    Bang,
    Not,
    PlusPlus,
    MinusMinus,
    Plus,
    Minus,
}

impl From<RawToken> for RawPrefixOperator {
    fn from(token: RawToken) -> Self {
        match token {
            Token![++] => Self::PlusPlus,
            Token![--] => Self::MinusMinus,
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
            RawPrefixOperator::PlusPlus => Token![++],
            RawPrefixOperator::MinusMinus => Token![--],
            RawPrefixOperator::Plus => Token![+],
            RawPrefixOperator::Minus => Token![-],
        }
    }
}

/// Represents a postfix operator with a specific span.
#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub struct PostfixOperator {
    pub span: Span,
    pub raw: RawPostfixOperator,
}

/// Represents a postfix operator.
#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub enum RawPostfixOperator {
    QuestionMark,
    PlusPlus,
    MinusMinus,
}

impl From<RawToken> for RawPostfixOperator {
    fn from(token: RawToken) -> Self {
        match token {
            Token![?] => Self::QuestionMark,
            Token![++] => Self::PlusPlus,
            Token![--] => Self::MinusMinus,
            _ => unreachable!(),
        }
    }
}

impl From<RawPostfixOperator> for RawToken {
    fn from(operator: RawPostfixOperator) -> Self {
        match operator {
            RawPostfixOperator::QuestionMark => Token![?],
            RawPostfixOperator::PlusPlus => Token![++],
            RawPostfixOperator::MinusMinus => Token![--],
        }
    }
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
    pub right: UntypedExpression,
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
    pub name: IdentifierAst,
    pub value: Option<UntypedExpression>,
}

impl UntypedExpression {
    /// Returns `true` if this expression has a block in it (except function expressions).
    /// Used to determine if this expression has to have semicolon at the end.
    /// Function expression do have blocks in them, but they must have a semicolon at the end.
    pub fn with_block(&self) -> bool {
        matches!(self, Self::If { .. } | Self::While { .. })
    }
}

/// Represents a statement.
#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    /// Defer statement
    ///
    /// ```txt
    /// defer file.close();
    /// ```
    Defer { call: UntypedExpression },

    /// Expression statement
    ///
    /// ```txt
    /// if x {
    ///     return Some("hello");
    /// }
    /// ```
    Expression {
        expression: UntypedExpression,
        has_semicolon: bool,
    },

    /// Break statement
    ///
    /// ```txt
    /// break;
    /// ```
    Break { span: Span },

    /// Continue statement
    ///
    /// ```txt
    /// continue;
    /// ```
    Continue { span: Span },

    /// Return statement
    ///
    /// ```txt
    /// /// Answer to the Ultimate Question of Life, the Universe, and Everything
    /// fun the_answer(): uint32 {
    ///     return 42;
    /// }
    /// ```
    Return { expression: UntypedExpression },

    /// Let statement
    ///
    /// ```txt
    /// let x = 1;
    /// ```
    Let {
        pattern: Pattern,
        value: Box<UntypedExpression>,
        ty: Option<TypeAst>,
    },
}

/// Represents a block of statements.
///
/// ```txt
/// fun main() { println!("Hello"); }
///            ^^^^^^^^^^^^^^^^^^^^^^ statements block
/// ```
pub type StatementsBlock = Vec<Statement>;

/// Represents a docstring.
pub type Docstring = Vec<String>;

#[derive(Debug, PartialEq, Clone)]
pub struct Documented<T> {
    value: T,
    docstring: Docstring,
}

pub trait WithDocComment {
    fn with_doc_comment(self, docstring: Docstring) -> Documented<Self>
    where
        Self: Sized,
    {
        Documented {
            value: self,
            docstring,
        }
    }
}

impl<T: Sized> WithDocComment for T {}

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
        visibility: Visibility,
        name: IdentifierAst,
        generic_parameters: Option<Vec<GenericParameter>>,
        where_clause: Option<WhereClause>,
        items: Vec<Documented<EnumItem>>,
    },

    /// Function item.
    ///
    /// ```txt
    /// fun foo() {
    ///     println("Hello")
    /// }
    /// ```
    Function(Function),

    /// Use item.
    ///
    /// ```txt
    /// use std.io;
    /// ```
    Use { visibility: Visibility, path: Path },

    /// Trait item.
    ///
    /// ```txt
    /// trait Into[T] {
    ///     fun into(self: Self) -> T;
    /// }
    /// ```
    Trait {
        visibility: Visibility,
        name: IdentifierAst,
        generic_parameters: Option<Vec<GenericParameter>>,
        where_clause: Option<WhereClause>,
        items: Vec<Documented<TraitItem>>,
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
        visibility: Visibility,
        generic_parameters: Option<Vec<GenericParameter>>,
        ty: TypeAst,
        r#trait: Option<TypeAst>,
        where_clause: Option<WhereClause>,
        items: Vec<Documented<TraitItem>>,
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
        name: IdentifierAst,
        generic_parameters: Option<Vec<GenericParameter>>,
        where_clause: Option<WhereClause>,
        fields: Vec<Documented<StructField>>,
    },

    /// Tuple-like struct item.
    ///
    /// ```txt
    /// struct MyStringWrapper(String);
    /// ```
    TupleLikeStruct {
        visibility: Visibility,
        name: IdentifierAst,
        generic_parameters: Option<Vec<GenericParameter>>,
        where_clause: Option<WhereClause>,
        fields: Vec<TupleField>,
    },

    /// Type alias item.
    TypeAlias(TypeAlias),
}

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
            Self::Impl => "impl",
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
    Just(IdentifierAst),
    Tuple {
        name: IdentifierAst,
        fields: Vec<TupleField>,
    },
    Struct {
        name: IdentifierAst,
        fields: Vec<Documented<StructField>>,
    },
}

/// Represents a tuple field.
///
/// ```txt
/// struct Test(pub String);
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct TupleField {
    pub visibility: Visibility,
    pub ty: TypeAst,
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
    pub visibility: Visibility,
    pub name: IdentifierAst,
    pub ty: TypeAst,
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
    pub visibility: Visibility,
    pub name: IdentifierAst,
    pub generic_parameters: Option<Vec<GenericParameter>>,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: Option<TypeAst>,
    pub where_clause: Option<WhereClause>,
    pub body: Option<StatementsBlock>,
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
    pub ty: Option<TypeAst>,
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
    pub name: IdentifierAst,
    pub ty: TypeAst,
    pub default_value: Option<UntypedExpression>,
}

/// Represents Ry source file.
#[derive(Debug, PartialEq, Clone)]
pub struct Module<'source> {
    pub file_id: FileID,
    pub source_file: &'source SourceFile<'source>,
    pub docstring: Docstring,
    pub items: Items,
}

pub type Items = Vec<Documented<Item>>;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Visibility(Option<Span>);

impl Visibility {
    pub fn private() -> Self {
        Self(None)
    }

    pub fn public(span: Span) -> Self {
        Self(Some(span))
    }

    pub fn span_of_pub(&self) -> Option<Span> {
        self.0
    }
}

impl Default for Visibility {
    fn default() -> Self {
        Self::private()
    }
}
