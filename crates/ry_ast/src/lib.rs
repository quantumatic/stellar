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

pub mod precedence;
pub mod span;
pub mod token;

use ry_interner::Symbol;
use span::{Span, Spanned};
use token::Token;

#[derive(Debug, PartialEq)]
pub enum Literal {
    Boolean(bool),
    Character(char),
    String(String),
    Integer(u64),
    Float(f64),
}

pub type Identifier = Spanned<Symbol>;

pub type Path = Spanned<Vec<Identifier>>;

#[derive(Debug, PartialEq)]
pub enum Pattern {
    Literal(Spanned<Literal>),
    Identifier {
        identifier: Identifier,
        pattern: Option<Box<Spanned<Pattern>>>,
    },
    Struct {
        r#struct: Path,
        fields: Vec<(Identifier, Option<Spanned<Pattern>>)>,
    },
    EnumItemTuple {
        r#enum: Path,
        inner_patterns: Vec<Spanned<Pattern>>,
    },
    Tuple {
        inner_patterns: Vec<Spanned<Pattern>>,
    },
    Path {
        path: Path,
    },
    Array {
        inner_patterns: Vec<Spanned<Pattern>>,
    },
    Grouped {
        inner: Box<Spanned<Pattern>>,
    },
    Or {
        left: Box<Spanned<Pattern>>,
        right: Box<Spanned<Pattern>>,
    },
    Rest, // ..
}

#[derive(Debug, PartialEq)]
pub enum Type {
    Array {
        element_type: Box<Spanned<Type>>,
    },
    Primary {
        path: Path,
        generic_arguments: Vec<Spanned<Type>>,
    },
    Tuple {
        element_types: Vec<Spanned<Type>>,
    },
    Function {
        parameter_types: Vec<Spanned<Type>>,
        return_type: Box<Spanned<Type>>,
    },
}

#[derive(Debug, PartialEq)]
pub struct GenericParameter {
    pub name: Identifier,
    pub constraint: Option<Spanned<Type>>,
}

pub type WhereClause = Vec<WhereClauseItem>;

#[derive(Debug, PartialEq)]
pub struct TypeAlias {
    pub visibility: Visibility,
    pub name: Identifier,
    pub generic_parameters: Vec<GenericParameter>,
    pub value: Option<Spanned<Type>>,
}

#[derive(Debug, PartialEq)]
pub struct WhereClauseItem {
    pub r#type: Spanned<Type>,
    pub constraint: Spanned<Type>,
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Array {
        elements: Vec<Spanned<Expression>>,
    },
    As {
        left: Box<Spanned<Expression>>,
        right: Spanned<Type>,
    },
    Binary {
        left: Box<Spanned<Expression>>,
        operator: Token,
        right: Box<Spanned<Expression>>,
    },
    StatementsBlock(StatementsBlock),
    Literal(Literal),
    Identifier(Symbol),
    If {
        if_blocks: Vec<(Spanned<Expression>, StatementsBlock)>,
        r#else: Option<StatementsBlock>,
    },
    Parenthesized {
        inner: Box<Spanned<Expression>>,
    },
    Property {
        left: Box<Spanned<Expression>>,
        right: Identifier,
    },
    Unary {
        inner: Box<Spanned<Expression>>,
        operator: Token,
        postfix: bool,
    },
    While {
        condition: Box<Spanned<Expression>>,
        body: StatementsBlock,
    },
    Call {
        left: Box<Spanned<Expression>>,
        arguments: Vec<Spanned<Expression>>,
    },
    GenericArguments {
        left: Box<Spanned<Expression>>,
        arguments: Vec<Spanned<Type>>,
    },
    Tuple {
        elements: Vec<Spanned<Expression>>,
    },
    Struct {
        left: Box<Spanned<Expression>>,
        fields: Vec<StructExpressionUnit>,
    },
    Let {
        left: Spanned<Pattern>,
        right: Box<Spanned<Expression>>,
    },
    Match {
        expression: Box<Spanned<Expression>>,
        block: Vec<MatchExpressionUnit>,
    },
    Function {
        parameters: Vec<FunctionParameter>,
        return_type: Option<Spanned<Type>>,
        block: StatementsBlock,
    },
}

#[derive(Debug, PartialEq)]
pub struct MatchExpressionUnit {
    pub left: Spanned<Pattern>,
    pub right: Spanned<Expression>,
}

#[derive(Debug, PartialEq)]
pub struct StructExpressionUnit {
    pub name: Identifier,
    pub value: Option<Spanned<Expression>>,
}

impl Expression {
    pub fn with_block(&self) -> bool {
        matches!(self, Self::If { .. } | Self::While { .. })
    }
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Defer {
        call: Spanned<Expression>,
    },
    Expression {
        expression: Spanned<Expression>,
        has_semicolon: bool,
    },
    Return {
        expression: Spanned<Expression>,
    },
}

pub type StatementsBlock = Vec<Statement>;

pub type Docstring = Vec<String>;

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub enum Item {
    Enum {
        visibility: Visibility,
        name: Identifier,
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
        name: Identifier,
        generic_parameters: Vec<GenericParameter>,
        where_clause: WhereClause,
        items: Vec<Documented<TraitItem>>,
    },
    Impl {
        visibility: Visibility,
        generic_parameters: Vec<GenericParameter>,
        r#type: Spanned<Type>,
        r#trait: Option<Spanned<Type>>,
        where_clause: WhereClause,
        items: Vec<Documented<TraitItem>>,
    },
    Struct {
        visibility: Visibility,
        name: Identifier,
        generic_parameters: Vec<GenericParameter>,
        where_clause: WhereClause,
        fields: Vec<Documented<StructField>>,
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

#[derive(Debug, PartialEq)]
pub enum EnumItem {
    Identifier(Identifier),
    Tuple {
        name: Identifier,
        fields: Vec<TupleField>,
    },
    Struct {
        name: Identifier,
        fields: Vec<Documented<StructField>>,
    },
}

#[derive(Debug, PartialEq)]
pub struct TupleField {
    pub visibility: Visibility,
    pub r#type: Spanned<Type>,
}

#[derive(Debug, PartialEq)]
pub struct StructField {
    pub visibility: Visibility,
    pub name: Identifier,
    pub r#type: Spanned<Type>,
}

#[derive(Debug, PartialEq)]
pub enum TraitItem {
    TypeAlias(TypeAlias),
    AssociatedFunction(AssociatedFunction),
}

#[derive(Debug, PartialEq)]
pub struct Function {
    pub visibility: Visibility,
    pub name: Identifier,
    pub generic_parameters: Vec<GenericParameter>,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: Option<Spanned<Type>>,
    pub where_clause: WhereClause,
    pub body: Option<StatementsBlock>,
}

pub type AssociatedFunction = Function;

#[derive(Debug, PartialEq)]
pub struct FunctionParameter {
    pub name: Identifier,
    pub r#type: Spanned<Type>,
    pub default_value: Option<Spanned<Expression>>,
}

/// Represents Ry source file.
#[derive(Debug, PartialEq)]
pub struct ProgramUnit {
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
