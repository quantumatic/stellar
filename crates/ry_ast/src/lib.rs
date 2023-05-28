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
pub mod visitor;

use ry_interner::Symbol;
use span::{Span, Spanned};
use std::collections::HashMap;
use token::Token;
use visitor::*;

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
    LiteralPattern(Literal),
    IdentifierPattern {
        identifier: Identifier,
        pattern: Option<Box<Pattern>>,
    },
    StructPattern {
        r#struct: Path,
        fields: HashMap<Identifier, Pattern>,
    },
    EnumItemTuplePattern {
        r#enum: Path,
        tuple_elements: Vec<Pattern>,
    },
    PathPattern {
        path: Path,
    },
    ArrayPattern {
        inner_patterns: Vec<Pattern>,
    },
    RestPattern, // ..
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
        variants: Vec<Documented<Identifier>>,
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
        members: Vec<Documented<StructMember>>,
    },
    TypeAlias(TypeAlias),
}

#[derive(Debug, PartialEq)]
pub struct StructMember {
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
