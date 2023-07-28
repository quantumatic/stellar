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

use ry_filesystem::location::Location;
use ry_interner::{PathID, Symbol};
use token::RawToken;

pub mod precedence;
pub mod serialize;
pub mod token;
pub mod visit;

pub type DefinitionIndex = usize;

/// An ID for every definition (module item) in a workspace.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct DefinitionID {
    pub index: DefinitionIndex,
    pub file_path_id: PathID,
}

/// A literal, e.g. `true`, `3`, `\"hello\"`.
#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    /// Boolean literal, e.g. `true` or `false`.
    Boolean { value: bool, location: Location },

    /// Character literal, e.g. `'a'`, `'\u{1234}'`.
    Character { value: char, location: Location },

    /// String literal, e.g. `"hello"`.
    String { value: String, location: Location },

    /// Integer literal, e.g. `123`,
    Integer { value: u64, location: Location },

    /// Float literal, e.g. `3.14`.
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

/// A symbol with a specified location, e.g. `foo`, `std`.
///
/// The reason why it's called [`IdentifierAST`] is because the name
/// [`Identifier`] already exists in the [`token`] module.
///
/// [`Identifier`]: ry_ast::token::Identifier
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct IdentifierAST {
    pub location: Location,
    pub symbol: Symbol,
}

/// A sequence of identifiers separated by `.`, e.g. `std.io`, `foo`.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Path {
    pub location: Location,
    pub identifiers: Vec<IdentifierAST>,
}

/// An import path, e.g. `std.io`, `std.io as myio`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ImportPath {
    pub path: Path,
    pub r#as: Option<IdentifierAST>,
}

/// A type constructor, e.g. `Option[T]`.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct TypeConstructor {
    pub location: Location,
    pub path: Path,
    pub type_arguments: Option<Vec<Type>>,
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

    /// A grouped pattern - surrounded by parentheses, e.g. `(a)`, `([1, .., 9])`.
    Grouped {
        location: Location,
        inner: Box<Self>,
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

/// A list of bounds (which are basically type constructors), e.g. `Debug + Into[T]`.
pub type Bounds = Vec<TypeConstructor>;

/// A type, e.g. `int32`, `(char): bool`, `(char, char)`.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Type {
    /// A type path, e.g. `char`, `Option[T]`.
    Constructor(TypeConstructor),

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

    /// A parenthesized type, e.g. `(int32)`.
    ///
    /// **Note**: parenthesized type is not a single element tuple type, because
    /// its syntax is: `(T,)`!
    Parenthesized {
        location: Location,
        inner: Box<Self>,
    },

    /// An interface object type, e.g. `dyn Iterator[Item = uint32]`, `dyn Debug + Clone`.
    InterfaceObject { location: Location, bounds: Bounds },
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
pub struct GenericParameter {
    pub name: IdentifierAST,
    pub bounds: Option<Bounds>,
    pub default_value: Option<Type>,
}

/// A type alias, e.g. `type MyResult = Result[String, MyError];`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TypeAlias {
    pub visibility: Visibility,
    pub name: IdentifierAST,
    pub generic_parameters: Option<Vec<GenericParameter>>,
    pub bounds: Option<Bounds>,
    pub value: Option<Type>,
    pub docstring: Option<String>,
}

/// A where clause predicate, e.g. `T: ToString`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WherePredicate {
    pub ty: Type,
    pub bounds: Bounds,
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

    /// Loop expression, e.g. `loop { ... }`
    Loop {
        location: Location,
        statements_block: StatementsBlock,
    },

    /// Binary expression, e.g. `1 + 2`.
    Binary {
        location: Location,
        left: Box<Self>,
        operator: BinaryOperator,
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

    /// Parenthesized expression, e.g. `(1 + 2)`.
    Parenthesized {
        location: Location,
        inner: Box<Self>,
    },

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
        operator: PrefixOperator,
    },

    /// Postfix expression, e.g. `safe_div(1, 0)?`, `a++`.
    Postfix {
        location: Location,
        inner: Box<Self>,
        operator: PostfixOperator,
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

    /// Type arguments expression, e.g. `sizeof[uint32]`.
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
        block: Vec<Statement>,
    },
}

/// A lambda function parameter, e.g. `x` in `|x| { x + 1 }`.
#[derive(Debug, PartialEq, Eq, Clone)]
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

/// A binary operator with a specific location.
///
/// See [`BinaryOperator`] for more information.
#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub struct BinaryOperator {
    pub location: Location,
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

/// A prefix operator with a specific location.
///
/// See [`PrefixOperator`] for more information.
#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub struct PrefixOperator {
    pub location: Location,
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

/// A postfix operator with a specific location.
///
/// See [`PostfixOperator`] for more information.
#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub struct PostfixOperator {
    pub location: Location,
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

/// A module item.
#[derive(Debug, PartialEq, Clone)]
pub enum ModuleItem {
    /// Enum item.
    Enum {
        visibility: Visibility,
        name: IdentifierAST,
        generic_parameters: Option<Vec<GenericParameter>>,
        where_predicates: Option<Vec<WherePredicate>>,
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
        generic_parameters: Option<Vec<GenericParameter>>,
        where_predicates: Option<Vec<WherePredicate>>,
        methods: Vec<Function>,
        implements: Option<Bounds>,
        docstring: Option<String>,
    },

    /// Struct item.
    Struct {
        visibility: Visibility,
        name: IdentifierAST,
        generic_parameters: Option<Vec<GenericParameter>>,
        where_predicates: Option<Vec<WherePredicate>>,
        fields: Vec<StructField>,
        methods: Vec<Function>,
        implements: Option<Bounds>,
        docstring: Option<String>,
    },

    /// Tuple-like struct item.
    TupleLikeStruct {
        visibility: Visibility,
        name: IdentifierAST,
        generic_parameters: Option<Vec<GenericParameter>>,
        where_predicates: Option<Vec<WherePredicate>>,
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
#[derive(Debug, PartialEq, Eq, Clone)]
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
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TupleField {
    pub visibility: Visibility,
    pub ty: Type,
}

/// A struct field, e.g. `name: String`, `pub age: uint32`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StructField {
    pub visibility: Visibility,
    pub name: IdentifierAST,
    pub ty: Type,
    pub docstring: Option<String>,
}

/// A function.
#[derive(Debug, PartialEq, Clone)]
pub struct Function {
    pub signature: FunctionSignature,
    pub body: Option<StatementsBlock>,
}

/// A function signature - information about function except a block.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FunctionSignature {
    pub visibility: Visibility,
    pub name: IdentifierAST,
    pub generic_parameters: Option<Vec<GenericParameter>>,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: Option<Type>,
    pub where_predicates: Option<Vec<WherePredicate>>,
    pub docstring: Option<String>,
}

/// A function parameter, e.g. `self`, `self: Self`, `a: uint32`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FunctionParameter {
    /// A function parameter that is not `self`.
    NotSelfParameter(NotSelfFunctionParameter),

    /// A self parameter.
    SelfParameter(SelfFunctionParameter),
}

/// A self parameter, e.g. `self`, `self: Self`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SelfFunctionParameter {
    pub self_location: Location,
    pub ty: Option<Type>,
}

/// A function parameter that is not `self`, e.g. `a: uint32`.
#[derive(Debug, PartialEq, Eq, Clone)]
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

/// A visibility qualifier - `pub` or nothing (private visibility).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Visibility(Option<Location>);

impl Visibility {
    #[inline]
    #[must_use]
    pub const fn private() -> Self {
        Self(None)
    }

    #[inline]
    #[must_use]
    pub const fn public(location: Location) -> Self {
        Self(Some(location))
    }

    #[inline]
    #[must_use]
    pub const fn location_of_pub(&self) -> Option<Location> {
        self.0
    }
}

impl Default for Visibility {
    fn default() -> Self {
        Self::private()
    }
}
