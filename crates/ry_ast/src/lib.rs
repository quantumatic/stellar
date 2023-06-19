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
use ry_source_file::span::Span;
use std::path;
use token::RawToken;

pub mod precedence;
pub mod token;

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Boolean { value: bool, span: Span },
    Character { value: char, span: Span },
    String { value: String, span: Span },
    Integer { value: u64, span: Span },
    Float { value: f64, span: Span },
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct IdentifierAst {
    pub span: Span,
    pub symbol: Symbol,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Path {
    pub span: Span,
    pub symbols: Vec<IdentifierAst>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Pattern {
    Literal(Literal),
    Identifier {
        span: Span,
        identifier: IdentifierAst,
        ty: Option<TypeAst>, // variable can already be initialized before
        pattern: Option<Box<Self>>,
    },
    Struct {
        span: Span,
        r#struct: Path,
        fields: Vec<StructFieldPattern>,
    },
    TupleLike {
        span: Span,
        r#enum: Path,
        inner_patterns: Vec<Self>,
    },
    Tuple {
        span: Span,
        inner_patterns: Vec<Self>,
    },
    Path {
        span: Span,
        path: Path,
    },
    List {
        span: Span,
        inner_patterns: Vec<Self>,
    },
    Grouped {
        span: Span,
        inner: Box<Self>,
    },
    Or {
        span: Span,
        left: Box<Self>,
        right: Box<Self>,
    },
    Rest {
        span: Span,
    }, // ..
}

impl Pattern {
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

#[derive(Debug, PartialEq, Clone)]
pub enum TypeAst {
    Constructor(TypeConstructorAst),
    Variable {
        span: Span,
        index: usize,
    },
    Tuple {
        span: Span,
        element_types: Vec<Self>,
    },
    Function {
        span: Span,
        parameter_types: Vec<Self>,
        return_type: Box<Self>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypeConstructorAst {
    pub span: Span,
    pub path: Path,
    pub generic_arguments: Vec<TypeAst>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct GenericParameter {
    pub name: IdentifierAst,
    pub constraint: Option<TypeAst>,
}

pub type WhereClause = Vec<WhereClauseItem>;

#[derive(Debug, PartialEq, Clone)]
pub struct TypeAlias {
    pub visibility: Visibility,
    pub name: IdentifierAst,
    pub generic_parameters: Vec<GenericParameter>,
    pub value: Option<TypeAst>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct WhereClauseItem {
    pub r#type: TypeAst,
    pub constraint: TypeAst,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    List {
        span: Span,
        elements: Vec<Self>,
    },
    As {
        span: Span,
        left: Box<Self>,
        right: TypeAst,
    },
    Binary {
        span: Span,
        left: Box<Self>,
        operator: BinaryOperator,
        right: Box<Self>,
    },
    StatementsBlock {
        span: Span,
        block: Vec<Statement>,
    },
    Literal(Literal),
    Identifier(IdentifierAst),
    Parenthesized {
        span: Span,
        inner: Box<Self>,
    },
    If {
        span: Span,
        if_blocks: Vec<(Self, Vec<Statement>)>,
        r#else: Option<Vec<Statement>>,
    },
    Property {
        span: Span,
        left: Box<Self>,
        right: IdentifierAst,
    },
    Prefix {
        span: Span,
        inner: Box<Self>,
        operator: PrefixOperator,
    },
    Postfix {
        span: Span,
        inner: Box<Self>,
        operator: PostfixOperator,
    },
    While {
        span: Span,
        condition: Box<Self>,
        body: Vec<Statement>,
    },
    Call {
        span: Span,
        left: Box<Self>,
        arguments: Vec<Self>,
    },
    GenericArguments {
        span: Span,
        left: Box<Self>,
        arguments: Vec<TypeAst>,
    },
    Tuple {
        span: Span,
        elements: Vec<Self>,
    },
    Struct {
        span: Span,
        left: Box<Self>,
        fields: Vec<StructExpressionUnit>,
    },
    Match {
        span: Span,
        expression: Box<Self>,
        block: Vec<MatchExpressionUnit>,
    },
    Function {
        span: Span,
        parameters: Vec<FunctionParameter>,
        return_type: Option<TypeAst>,
        block: Vec<Statement>,
    },
}

impl Expression {
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
            | Self::Property { span, .. }
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

#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub struct BinaryOperator {
    pub span: Span,
    pub raw: RawBinaryOperator,
}

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
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub struct PrefixOperator {
    pub span: Span,
    pub raw: RawPrefixOperator,
}

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

#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub struct PostfixOperator {
    pub span: Span,
    pub raw: RawPostfixOperator,
}

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

#[derive(Debug, PartialEq, Clone)]
pub struct MatchExpressionUnit {
    pub left: Pattern,
    pub right: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructExpressionUnit {
    pub name: IdentifierAst,
    pub value: Option<Expression>,
}

impl Expression {
    pub fn with_block(&self) -> bool {
        matches!(self, Self::If { .. } | Self::While { .. })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Defer {
        call: Expression,
    },
    Expression {
        expression: Expression,
        has_semicolon: bool,
    },
    Return {
        expression: Expression,
    },
    Let {
        pattern: Pattern,
        value: Box<Expression>,
        ty: Option<TypeAst>,
    },
}

pub type StatementsBlock = Vec<Statement>;

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

#[derive(Debug, PartialEq, Clone)]
pub enum Item {
    Enum {
        visibility: Visibility,
        name: IdentifierAst,
        generic_parameters: Vec<GenericParameter>,
        where_clause: WhereClause,
        items: Vec<Documented<EnumItem>>,
    },
    Function(Function),
    Import {
        visibility: Visibility,
        path: Path,
    },
    Trait {
        visibility: Visibility,
        name: IdentifierAst,
        generic_parameters: Vec<GenericParameter>,
        where_clause: WhereClause,
        items: Vec<Documented<TraitItem>>,
    },
    Impl {
        visibility: Visibility,
        generic_parameters: Vec<GenericParameter>,
        r#type: TypeAst,
        r#trait: Option<TypeAst>,
        where_clause: WhereClause,
        items: Vec<Documented<TraitItem>>,
    },
    Struct {
        visibility: Visibility,
        name: IdentifierAst,
        generic_parameters: Vec<GenericParameter>,
        where_clause: WhereClause,
        fields: Vec<Documented<StructField>>,
    },
    TupleLikeStruct {
        visibility: Visibility,
        name: IdentifierAst,
        generic_parameters: Vec<GenericParameter>,
        where_clause: WhereClause,
        fields: Vec<TupleField>,
    },
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

#[derive(Debug, PartialEq, Clone)]
pub struct TupleField {
    pub visibility: Visibility,
    pub r#type: TypeAst,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructField {
    pub visibility: Visibility,
    pub name: IdentifierAst,
    pub r#type: TypeAst,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TraitItem {
    TypeAlias(TypeAlias),
    AssociatedFunction(AssociatedFunction),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Function {
    pub visibility: Visibility,
    pub name: IdentifierAst,
    pub generic_parameters: Vec<GenericParameter>,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: Option<TypeAst>,
    pub where_clause: WhereClause,
    pub body: Option<StatementsBlock>,
}

pub type AssociatedFunction = Function;

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionParameter {
    pub name: IdentifierAst,
    pub r#type: TypeAst,
    pub default_value: Option<Expression>,
}

/// Represents Ry source file.
#[derive(Debug, PartialEq, Clone)]
pub struct Module<'a> {
    pub filepath: &'a path::Path,
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
